//! Job de indexação: varre o histórico do repositório recém-aberto e enche o índice de busca
//! em background — nunca atrasa a resposta de abrir, nunca compete pelo mesmo lock que o log
//! (cada leitura abre seu próprio `Git2Repo`, e o SQLite está em WAL).
//!
//! Passo 42 é só isto: o job e o indicador. A busca de verdade sobre o que fica indexado aqui
//! é o Passo 43.

use std::{path::PathBuf, sync::Arc};

use porc_git::read::{Git2Repo, RepoRead};
use porc_index::{commits::CommitRow, Index};

use crate::{
    jobs::{JobHandle, Progress},
    AppState,
};

/// A cada quantos commits o progresso é republicado. Um evento por commit em 100k commits
/// seria 100k mensagens de WebSocket — o indicador não precisa de granularidade fina, só de
/// mostrar que está andando.
const PROGRESS_EVERY: usize = 500;

/// Confere se o repositório já está com o índice em dia e, se não, dispara o job. Fogo e
/// esquece de propósito: quem acabou de abrir o repositório não deveria esperar a indexação
/// para ver o log — é exatamente o "sem bloquear" do aceite deste passo.
pub fn maybe_spawn(state: &AppState, repo_id: String, path: PathBuf, head_oid: Option<String>) {
    let Some(head_oid) = head_oid else {
        // `unborn`: sem commit nenhum, nada para indexar.
        return;
    };

    tokio::spawn(run(state.clone(), repo_id, path, head_oid));
}

async fn run(state: AppState, repo_id: String, path: PathBuf, head_oid: String) {
    let index = state.index.clone();
    let check_repo_id = repo_id.clone();
    let check = tokio::task::spawn_blocking(move || index.indexed_tip(&check_repo_id)).await;

    match check {
        // Já indexado até este `HEAD`: nada mudou desde a última vez, pular o job inteiro é
        // o que faz reabrir o mesmo repositório repetidas vezes não gerar trabalho à toa.
        Ok(Ok(Some(tip))) if tip == head_oid => return,
        Ok(Ok(_)) => {}
        Ok(Err(err)) => {
            tracing::warn!(%err, "não consegui consultar o índice antes de decidir indexar");
        }
        Err(err) => {
            tracing::error!(%err, "spawn_blocking falhou ao consultar o índice");
            return;
        }
    }

    // Teto de jobs concorrentes batido, ou geração de id falhou: indexação não é essencial o
    // bastante para brigar por vaga com um clone ou um fetch de verdade — ela tenta de novo no
    // próximo repositório aberto.
    let Ok(handle) = state.jobs.create("index") else {
        tracing::debug!(repo_id, "sem vaga para o job de indexação agora");
        return;
    };

    handle.log(format!("indexando {}", path.display()));

    let index = state.index.clone();
    let outcome = tokio::task::spawn_blocking(move || {
        index_blocking(handle, &index, &repo_id, &path, &head_oid)
    })
    .await;

    if let Err(err) = outcome {
        tracing::error!(%err, "spawn_blocking falhou ao indexar");
    }
}

/// Faz o trabalho de verdade. Bloqueante: libgit2 e SQLite, os dois só valem dentro de
/// `spawn_blocking`. `handle` é consumida aqui — ela termina o job antes de devolver.
fn index_blocking(
    handle: JobHandle,
    index: &Arc<Index>,
    repo_id: &str,
    path: &std::path::Path,
    head_oid: &str,
) {
    let repo = match Git2Repo::open(path) {
        Ok(repo) => repo,
        Err(err) => return handle.fail(err.to_string()),
    };

    // Tudo em memória até o fim, não gravado aos pedaços: `replace_commits` troca o índice
    // inteiro numa transação só, e uma indexação cancelada ou que falhe no meio não pode
    // deixar o repositório com metade dos commits indexados. Para 100k commits isto é da
    // ordem de poucas dezenas de MB — nada que valha a complicação de gravar em lotes.
    let mut rows = Vec::new();
    let mut cancelled = false;

    let walked = repo.walk_for_index(&mut |entry| {
        if handle.is_cancelled() {
            cancelled = true;
            return false;
        }

        rows.push(CommitRow {
            oid: entry.oid,
            author: entry.author,
            email: entry.email,
            time: entry.time,
            summary: entry.summary,
        });

        if rows.len().is_multiple_of(PROGRESS_EVERY) {
            handle.progress(Progress {
                phase: "indexando".to_owned(),
                // Contagem total do repositório não é conhecida de graça (custaria outro
                // revwalk inteiro); o indicador mostra o que já foi feito, não uma barra.
                fraction: None,
                detail: Some(format!("{} commits", rows.len())),
            });
        }

        true
    });

    if cancelled {
        return handle.cancelled();
    }

    match walked {
        Ok(_) => {}
        Err(err) => return handle.fail(err.to_string()),
    }

    let count = rows.len();
    if let Err(err) = index.replace_commits(repo_id, head_oid, &rows) {
        return handle.fail(err.to_string());
    }

    handle.log(format!("{count} commits indexados"));
    handle.done(serde_json::json!({ "commits": count }));
}
