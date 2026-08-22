//! Parser dos registros `%x00`-separados de `git log -z --format=…`.
//!
//! Cinco campos por commit (`%H%x00%an%x00%ae%x00%at%x00%s`), cada um terminado em NUL —
//! inclusive o último, porque `-z` troca o `\n` que o git poria entre commits por outro NUL. O
//! byte stream inteiro é só uma repetição de "campo\0" sem separador nenhum a mais para
//! confundir; é por isso que cortar em `\0` sem estado nenhum sobre "onde começa um commit" já
//! basta — quem conta os campos é o [`RecordSplitter`].
//!
//! Usado pelo filtro de caminho (Passo 45) e pela busca de conteúdo (Passo 46) — os dois são
//! streaming de `git log`, só o argumento que muda.

/// O que este parser devolve: os mesmos campos que `porc_index::commits::SearchHit` mostra na
/// UI, mas sem depender do índice — paths e conteúdo não são indexados (decisão do
/// `CLAUDE.md`), então isto vem direto do `git log` streaming, não do SQLite.
#[derive(Debug, Clone, PartialEq)]
pub struct StreamedCommit {
    pub oid: String,
    pub author: String,
    pub email: String,
    pub time: i64,
    pub summary: String,
}

const FIELDS_PER_COMMIT: usize = 5;

#[derive(Debug, Default)]
pub struct RecordSplitter {
    /// Bytes do campo em andamento, ainda sem o NUL que o fecha.
    buffer: Vec<u8>,
    /// Campos já fechados deste commit, esperando os que faltam.
    fields: Vec<String>,
}

impl RecordSplitter {
    /// Consome um pedaço do stdout e devolve os commits que ele fechou. Leitura de pipe não
    /// respeita fronteira de campo nem de commit — um `push` pode fechar vários dos dois, ou
    /// nenhum.
    pub fn push(&mut self, chunk: &[u8]) -> Vec<StreamedCommit> {
        self.buffer.extend_from_slice(chunk);

        let mut commits = Vec::new();

        while let Some(at) = self.buffer.iter().position(|&byte| byte == 0) {
            // `from_utf8_lossy`: um nome de autor ou uma mensagem de commit fora de UTF-8 não
            // pode derrubar o streaming inteiro — mesma escolha do resto do `porc-git`.
            let field = String::from_utf8_lossy(&self.buffer[..at]).into_owned();
            self.buffer.drain(..=at);
            self.fields.push(field);

            if self.fields.len() == FIELDS_PER_COMMIT {
                let mut fields = std::mem::take(&mut self.fields).into_iter();
                commits.push(StreamedCommit {
                    oid: fields.next().unwrap_or_default(),
                    author: fields.next().unwrap_or_default(),
                    email: fields.next().unwrap_or_default(),
                    time: fields.next().and_then(|t| t.parse().ok()).unwrap_or(0),
                    summary: fields.next().unwrap_or_default(),
                });
            }
        }

        commits
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn record(oid: &str, author: &str, email: &str, time: &str, summary: &str) -> Vec<u8> {
        [oid, author, email, time, summary]
            .iter()
            .flat_map(|field| field.bytes().chain(std::iter::once(0)))
            .collect()
    }

    #[test]
    fn um_chunk_com_varios_commits() {
        let mut bytes = record("aaa", "Ana", "ana@example.com", "1700000000", "primeiro");
        bytes.extend(record(
            "bbb",
            "Bia",
            "bia@example.com",
            "1700000001",
            "segundo",
        ));

        let mut splitter = RecordSplitter::default();
        let commits = splitter.push(&bytes);

        assert_eq!(commits.len(), 2);
        assert_eq!(commits[0].oid, "aaa");
        assert_eq!(commits[0].time, 1_700_000_000);
        assert_eq!(commits[1].oid, "bbb");
        assert_eq!(commits[1].summary, "segundo");
    }

    #[test]
    fn commit_partido_no_meio_de_um_campo_espera_o_resto() {
        let bytes = record("aaa", "Ana", "ana@example.com", "1700000000", "primeiro");

        let mut splitter = RecordSplitter::default();
        // Corta bem no meio do campo de e-mail — leitura de pipe não respeita fronteira nenhuma.
        let mid = bytes.iter().position(|&b| b == 0).unwrap() + 6;

        assert!(splitter.push(&bytes[..mid]).is_empty());
        let commits = splitter.push(&bytes[mid..]);

        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].oid, "aaa");
        assert_eq!(commits[0].email, "ana@example.com");
    }

    #[test]
    fn campo_de_data_ilegivel_vira_zero_em_vez_de_derrubar_o_commit() {
        let bytes = record("aaa", "Ana", "ana@example.com", "nao-e-numero", "x");

        let mut splitter = RecordSplitter::default();
        let commits = splitter.push(&bytes);

        assert_eq!(commits.len(), 1);
        assert_eq!(commits[0].time, 0);
    }

    #[test]
    fn nada_sobra_sem_nul_no_final() {
        let mut splitter = RecordSplitter::default();
        // Quatro campos completos, o quinto sem o NUL que fecharia o commit.
        let mut bytes = record("aaa", "Ana", "ana@example.com", "1700000000", "x");
        bytes.pop();

        assert!(splitter.push(&bytes).is_empty());
    }
}
