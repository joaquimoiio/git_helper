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

/// A que tipo de ponta uma `RefMarker` pertence.
///
/// Só o que dá para marcar numa linha de log: branch local, branch remota ou tag. `Head` é o
/// caso à parte de um `HEAD` **destacado** — sem branch nenhuma para carregar o marcador, a
/// própria ponta é "HEAD".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RefKind {
    Branch,
    Remote,
    Tag,
    Head,
}

/// Uma ponta (branch, tag, ou o `HEAD` destacado) e o commit para o qual ela aponta.
///
/// Peso de apresentação, não de leitura: nada aqui diz em que lane ou linha do log a ponta
/// cai — quem cruza `commit` com o oid da linha é o cliente, porque a lista de refs (dezenas,
/// no máximo centenas) não muda por página do log e não vale a pena repetir a cada uma.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefMarker {
    /// Nome curto: `main`, `origin/main`, `v1.0` — nunca o `refs/heads/…` inteiro.
    pub name: String,
    pub kind: RefKind,
    pub commit: String,
    /// `true` quando é a ponta para a qual o `HEAD` aponta agora.
    pub is_head: bool,
}

/// Uma linha do log, já com a coluna do grafo.
///
/// `lane` é a coluna desta linha; `parent_lanes` é a coluna para onde cada aresta desce, na
/// mesma ordem de `parents`. `None` num `parent_lanes` é a fronteira de um clone raso — o pai
/// existe na mensagem do commit mas não no odb local, e a linha não continua para lugar
/// nenhum. Não confundir com a aresta que sai da **página** carregada: essa o cliente decide
/// sozinho, comparando o oid do pai com os commits que já tem — o servidor não sabe o que o
/// cliente já buscou.
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
    pub lane: usize,
    pub parent_lanes: Vec<Option<usize>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogPage {
    pub commits: Vec<Commit>,
    /// `None` na última página. Opaco para quem consome: volta como veio.
    pub next_cursor: Option<String>,
}

/// Nome, e-mail e quando — de quem escreveu o commit ou de quem o aplicou. São a mesma forma
/// porque o git as guarda da mesma forma; só o significado muda (autor vs. committer).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Signature {
    pub name: String,
    pub email: String,
    /// Em segundos desde a época.
    pub time: i64,
    /// Em minutos, para a UI mostrar a hora local de quem assinou.
    pub offset: i32,
}

/// O que aconteceu com um arquivo neste commit, contra o primeiro pai.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum FileChangeKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Typechange,
}

/// Um arquivo tocado pelo commit, com a contagem de linhas — não o patch em si. O patch
/// (hunks estruturados) é do Passo 41; aqui é só o que um resumo de commit mostra.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileChange {
    /// Caminho atual. Em `Deleted`, o caminho que o arquivo tinha antes de sumir.
    pub path: String,
    /// Só presente em `Renamed`/`Copied`, e só quando difere de `path`.
    pub old_path: Option<String>,
    pub kind: FileChangeKind,
    pub insertions: usize,
    pub deletions: usize,
    /// `true` quando o git marcou o arquivo como binário — contagem de linhas não tem sentido.
    pub binary: bool,
}

/// Detalhe completo de um commit: o que o painel direito do log mostra ao selecionar uma
/// linha.
///
/// O diff é contra o **primeiro pai** (contra a árvore vazia, na raiz) — é a mesma
/// simplificação que `git show` faz por padrão num merge, sem `-m`. Combinar os dois lados de
/// um merge é decisão de apresentação para revisitar se fizer falta na prática.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommitDetail {
    pub oid: String,
    pub parents: Vec<String>,
    pub author: Signature,
    pub committer: Signature,
    /// Mensagem completa — resumo (primeira linha) e corpo, como o git guarda.
    pub message: String,
    pub insertions: usize,
    pub deletions: usize,
    pub files: Vec<FileChange>,
}

/// O que uma linha do hunk é, para a UI decidir cor e coluna sem reler o `origin` do git.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum DiffLineKind {
    Context,
    Addition,
    Deletion,
}

/// Uma linha dentro de um hunk. Sem o `\n` final — é apresentação, a UI decide como quebrar.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffLine {
    pub kind: DiffLineKind,
    /// Número no arquivo antigo. `None` em `Addition` — a linha não existia lá.
    pub old_lineno: Option<u32>,
    /// Número no arquivo novo. `None` em `Deletion` — a linha não existe mais aqui.
    pub new_lineno: Option<u32>,
    pub content: String,
}

/// Um hunk: um trecho contíguo de mudança, com um pouco de contexto ao redor.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiffHunk {
    /// A linha `@@ -a,b +c,d @@ …` que o git gera, para quem já reconhece o formato.
    pub header: String,
    pub lines: Vec<DiffLine>,
}

/// O diff de **um** arquivo dentro de um commit — não o commit inteiro. Um commit pode tocar
/// centenas de arquivos; mandar todos os hunks de uma vez seria o oposto de "sob demanda".
///
/// Três formas, não hunks-mais-uma-flag: `Binary` e `NotUtf8` não têm o que um hunk mostraria,
/// e um enum errado de compilar do que um `Vec` vazio que a UI teria que adivinhar o motivo.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum FileDiff {
    Text {
        hunks: Vec<DiffHunk>,
    },
    /// O git marcou o arquivo como binário (heurística dele, a mesma do `FileChange.binary`).
    Binary,
    /// Texto de verdade, mas alguma linha não é UTF-8 válido — um encoding legado, não binário.
    /// Mostrar mojibake (`from_utf8_lossy`) seria pior que avisar que não dá para ler.
    NotUtf8,
}

/// Um slot de lane no cursor: livre, ou esperando um oid (em hex) específico como próximo
/// commit daquela coluna.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaneSlot {
    Free,
    Waiting(String),
}

/// Prefixo de versão do cursor. Existe para um cursor de formato antigo não ser lido como
/// novo quando o formato muda — o `v1` só carregava a fronteira do revwalk; o `v2` (Passo 37)
/// carrega o estado das lanes inteiro, porque é o que permite à próxima página continuar o
/// desenho do grafo sem recalculá-lo desde o início.
const CURSOR_PREFIX: &str = "v2.";

/// Serializa o estado das lanes num cursor opaco.
///
/// Não é só a fronteira do revwalk (os oids ainda por emitir): é a posição de **cada** lane,
/// livre ou ocupada, porque é essa posição que decide em qual coluna a próxima página aloca
/// uma lane nova. Sem o estado completo, duas lanes livres no meio do desenho poderiam trocar
/// de ordem entre uma página e a seguinte — o mesmo grafo, mas desenhado diferente dos dois
/// lados da emenda.
///
/// Lanes livres **no final** do vetor são cortadas: elas não mudam nenhuma decisão futura (uma
/// lane nova aloca no primeiro slot livre, ou no fim se não houver nenhum — os dois dão no
/// mesmo se o fim já é livre), só o tamanho do cursor. Lanes livres **no meio** ficam: a
/// posição delas é exatamente o que faz a busca por "primeiro slot livre" ser determinística.
///
/// Hex separado por ponto, e não base64: são caracteres seguros em query string, e não vale
/// somar uma dependência para isso.
pub fn encode_cursor(lanes: &[LaneSlot]) -> String {
    let trimmed = match lanes
        .iter()
        .rposition(|slot| matches!(slot, LaneSlot::Waiting(_)))
    {
        Some(last) => &lanes[..=last],
        None => &[],
    };

    let body = trimmed
        .iter()
        .map(|slot| match slot {
            LaneSlot::Free => "-".to_owned(),
            LaneSlot::Waiting(oid) => oid.clone(),
        })
        .collect::<Vec<_>>()
        .join(".");

    format!("{CURSOR_PREFIX}{body}")
}

/// `None` quando o cursor não é deste formato. Quem valida se os oids **existem** é o
/// `read`, que é quem tem o repositório na mão.
pub fn decode_cursor(cursor: &str) -> Option<Vec<LaneSlot>> {
    let body = cursor.strip_prefix(CURSOR_PREFIX)?;

    // Corpo vazio nunca é emitido (cursor sem nenhuma lane esperando nada é a última página,
    // e ela vem sem cursor), então recebê-lo de volta é sinal de cursor adulterado.
    if body.is_empty() {
        return None;
    }

    Some(
        body.split('.')
            .map(|part| match part {
                "-" => LaneSlot::Free,
                oid => LaneSlot::Waiting(oid.to_owned()),
            })
            .collect(),
    )
}

/// O que aconteceu com um caminho no `status` do working tree.
///
/// `Unmerged` é conflito (linha `u` do porcelain v2) — não é "staged" nem "unstaged" no
/// sentido normal, mas o Passo 47 não abre um quarto grupo para isso: fica em `unstaged`,
/// visível, e o Bloco G decide o que fazer com ele.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum StatusKind {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Typechange,
    Unmerged,
    Untracked,
}

/// Um caminho dentro de um dos três grupos do status.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StatusEntry {
    pub path: String,
    /// Só presente em `Renamed`/`Copied`.
    pub old_path: Option<String>,
    pub kind: StatusKind,
}

/// O cabeçalho `# branch.*` do porcelain v2 — quem é o `HEAD` e como ele se compara ao
/// upstream, sem abrir um segundo request só para isso.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchStatus {
    /// `None` em repositório sem commit nenhum (`branch.oid` vem `(initial)`).
    pub oid: Option<String>,
    /// `None` com `HEAD` destacado — o oid já diz tudo que há para dizer nesse caso.
    pub head: Option<String>,
    pub detached: bool,
    /// Ausente quando a branch não tem upstream configurado.
    pub upstream: Option<String>,
    pub ahead: u32,
    pub behind: u32,
}

/// Operação em andamento que ainda não terminou — o que falta saber antes de um `commit`
/// normal (Bloco E) ser seguro, e o que o Bloco G usa para desenhar a barra de "resolvendo
/// conflitos". Vem do `git2::Repository::state()`, não do `status` em si: as duas fontes são
/// baratas, mas são chamadas diferentes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum RepoState {
    Clean,
    Merge,
    Rebase,
    CherryPick,
    Revert,
    Bisect,
}

/// O status completo do working tree — o que a rota `GET /repos/{id}/status` devolve.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorktreeStatus {
    pub branch: BranchStatus,
    pub state: RepoState,
    pub staged: Vec<StatusEntry>,
    pub unstaged: Vec<StatusEntry>,
    pub untracked: Vec<StatusEntry>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_vai_e_volta() {
        let lanes = vec![
            LaneSlot::Waiting("0b61c6f0000000000000000000000000000000aa".to_owned()),
            LaneSlot::Free,
            LaneSlot::Waiting("2cea64a0000000000000000000000000000000bb".to_owned()),
        ];

        let cursor = encode_cursor(&lanes);
        assert_eq!(decode_cursor(&cursor), Some(lanes));
    }

    #[test]
    fn lanes_livres_do_final_sao_cortadas_mas_nao_as_do_meio() {
        let lanes = vec![
            LaneSlot::Waiting("0b61c6f0000000000000000000000000000000aa".to_owned()),
            LaneSlot::Free,
            LaneSlot::Free,
        ];

        let cursor = encode_cursor(&lanes);
        assert_eq!(
            decode_cursor(&cursor),
            Some(vec![LaneSlot::Waiting(
                "0b61c6f0000000000000000000000000000000aa".to_owned()
            )])
        );
    }

    #[test]
    fn cursor_de_outro_formato_nao_decodifica() {
        for cursor in ["", "v2.", "v1.0b61c6f", "lixo"] {
            assert_eq!(decode_cursor(cursor), None, "cursor {cursor:?}");
        }
    }
}
