//! Modelos do domínio git, já no formato em que a UI os consome.
//!
//! Derivam `Serialize` aqui, e não numa camada de DTO no `porc-server`: um espelho de cada
//! struct só para trocar de crate seria trabalho de manutenção sem informação nova. Serializar
//! não é conhecer HTTP — o crate continua sem saber o que é uma rota.

use serde::Serialize;

/// Estado do `HEAD`.
///
/// São três estados de verdade, não dois com um caso de erro: `Unborn` é o repositório
/// recém-criado, que tem branch mas ainda não tem commit. Tratá-lo como erro obrigaria toda a
/// UI a lidar com "repo aberto mas quebrado" no minuto seguinte a um `git init`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum Head {
    Branch { name: String, commit: String },
    Detached { commit: String },
    Unborn { name: String },
}

impl Head {
    /// Nome para mostrar na barra de topo e no título da aba. Em detached, o hash curto — é o
    /// que o `git status` mostra, e é o que o usuário reconhece.
    pub fn label(&self) -> String {
        match self {
            Head::Branch { name, .. } | Head::Unborn { name } => name.clone(),
            Head::Detached { commit } => commit.chars().take(7).collect(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RepoInfo {
    /// Raiz da worktree. Em repositório bare não há worktree, e aqui vai o gitdir.
    pub path: String,
    /// Nome curto para a UI: última componente do caminho, sem o `.git` de um bare.
    pub name: String,
    pub bare: bool,
    /// `true` quando `HEAD` aponta para um commit em vez de uma branch.
    pub detached: bool,
    pub head: Head,
    /// Já resolvido do `Head`, para a UI não repetir o `match` em três lugares.
    pub branch: String,
}

/// Uma linha do log.
///
/// Sem grafo ainda: lanes e arestas entram no Passo 37, calculadas no servidor e enviadas
/// prontas. Aqui é só o que a linha de texto precisa mostrar.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Commit {
    /// Hash completo. Abreviar é decisão de apresentação, e abreviar com garantia de unicidade
    /// custaria uma consulta ao odb por commit — 500 por página, para nada.
    pub oid: String,
    /// Em ordem: o primeiro pai é a linha principal. Mais de um significa merge.
    pub parents: Vec<String>,
    pub author: String,
    pub email: String,
    /// Data do **autor**, em segundos desde a época.
    pub time: i64,
    /// Fuso do autor em minutos, para a UI poder mostrar a hora local de quem escreveu.
    pub offset: i32,
    /// Primeira linha da mensagem.
    pub summary: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogPage {
    pub commits: Vec<Commit>,
    /// `None` na última página. Opaco para quem consome: volta como veio.
    pub next_cursor: Option<String>,
}

/// Prefixo de versão do cursor. Existe para um cursor velho não ser lido como novo quando o
/// formato mudar — no Passo 37 ele passa a carregar também o estado das lanes.
const CURSOR_PREFIX: &str = "v1.";

/// Serializa a fronteira do revwalk num cursor opaco.
///
/// A fronteira é o conjunto de commits já descobertos e ainda não emitidos no ponto de corte.
/// É ela que faz a página seguinte custar o mesmo que a primeira: em vez de recomeçar do topo
/// e descartar o que já foi mandado, o revwalk é empurrado direto para onde parou.
///
/// Hex separado por ponto, e não base64: são caracteres seguros em query string, e não vale
/// somar uma dependência antes de o cursor precisar carregar bytes que não são oid.
pub fn encode_cursor(frontier: &[String]) -> String {
    format!("{CURSOR_PREFIX}{}", frontier.join("."))
}

/// `None` quando o cursor não é deste formato. Quem valida se os oids **existem** é o
/// `read`, que é quem tem o repositório na mão.
pub fn decode_cursor(cursor: &str) -> Option<Vec<String>> {
    let frontier = cursor.strip_prefix(CURSOR_PREFIX)?;

    // Fronteira vazia nunca é emitida (página sem fronteira é a última, e vem sem cursor),
    // então recebê-la de volta é sinal de cursor adulterado.
    if frontier.is_empty() {
        return None;
    }

    Some(frontier.split('.').map(str::to_owned).collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_vai_e_volta() {
        let frontier = vec![
            "0b61c6f0000000000000000000000000000000aa".to_owned(),
            "2cea64a0000000000000000000000000000000bb".to_owned(),
        ];

        let cursor = encode_cursor(&frontier);
        assert_eq!(decode_cursor(&cursor), Some(frontier));
    }

    #[test]
    fn cursor_de_outro_formato_nao_decodifica() {
        for cursor in ["", "v1.", "lixo", "v2.0b61c6f"] {
            assert_eq!(decode_cursor(cursor), None, "cursor {cursor:?}");
        }
    }
}
