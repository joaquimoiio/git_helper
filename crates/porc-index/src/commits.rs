//! Commits indexados e a busca por mensagem/autor (FTS5).
//!
//! `walk_for_index` (`porc-git`) enche a tabela `commits`; `commits_fts` é uma tabela virtual
//! **à parte**, sincronizada à mão dentro da mesma transação de `replace_commits` — sem
//! `content=` externo nem gatilho, porque a substituição já é total a cada reindexação (Passo
//! 42), então "recriar os dois juntos" é mais simples que manter os dois em sincronia via
//! trigger.

use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::params;
use serde::Serialize;

use crate::{Index, IndexError};

pub(crate) const SCHEMA: &str = "
CREATE TABLE IF NOT EXISTS commits (
    repo_id TEXT NOT NULL,
    oid     TEXT NOT NULL,
    author  TEXT NOT NULL,
    email   TEXT NOT NULL,
    ts      INTEGER NOT NULL,
    summary TEXT NOT NULL,
    PRIMARY KEY (repo_id, oid)
);
CREATE TABLE IF NOT EXISTS index_state (
    repo_id     TEXT PRIMARY KEY,
    indexed_tip TEXT NOT NULL,
    indexed_at  INTEGER NOT NULL
);
CREATE VIRTUAL TABLE IF NOT EXISTS commits_fts USING fts5(
    summary,
    author,
    email UNINDEXED,
    ts UNINDEXED,
    oid UNINDEXED,
    repo_id UNINDEXED
);
";

/// Uma linha pronta para inserir — o que a varredura de `porc-git` devolve, sem nada de HTTP.
#[derive(Debug, Clone)]
pub struct CommitRow {
    pub oid: String,
    pub author: String,
    pub email: String,
    pub time: i64,
    pub summary: String,
}

/// Um resultado de busca — o bastante para desenhar uma linha sem depender de o commit já
/// estar numa página do log carregada pelo cliente. Sem `lane`: a lista de resultados é um
/// flat list à parte do grafo, não uma sub-página do log.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchHit {
    pub oid: String,
    pub author: String,
    pub email: String,
    pub time: i64,
    pub summary: String,
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis() as i64)
        .unwrap_or(0)
}

/// Transforma a caixa de busca crua numa expressão FTS5 segura.
///
/// Cada palavra vira uma frase entre aspas (`"palavra"`) — é o que faz um `AND`, um `-` ou um
/// `"` soltos no que a pessoa digitou não virarem sintaxe de operador e derrubarem a consulta.
/// A última palavra ganha `*` (prefixo): é o que faz a letra que acabou de ser digitada já
/// filtrar, em vez de esperar a palavra inteira — essencial para "incremental a cada tecla".
/// `None` para entrada vazia: caixa de busca limpa é "sem filtro", não "filtra por nada".
fn build_match_query(raw: &str) -> Option<String> {
    let words: Vec<&str> = raw.split_whitespace().collect();
    if words.is_empty() {
        return None;
    }

    let escape = |word: &str| format!("\"{}\"", word.replace('"', "\"\""));

    let mut parts: Vec<String> = words[..words.len() - 1].iter().map(|w| escape(w)).collect();
    parts.push(format!("{}*", escape(words[words.len() - 1])));

    Some(parts.join(" "))
}

/// `true` para um único token hex de 4 a 40 caracteres — o formato de um hash de commit
/// (curto ou completo). Quando a caixa de busca inteira é isto, a consulta pula a análise de
/// sintaxe unificada e vai direto ao prefixo de oid — "com índice", não busca textual. Uma
/// palavra que por acaso só usa `a-f0-9` (`"cafe"`, `"deed"`) cai aqui também; é a mesma
/// ambiguidade que o próprio `git` tem com hashes curtos, não algo para resolver aqui.
fn is_hash_like(query: &str) -> bool {
    (4..=40).contains(&query.len()) && query.bytes().all(|b| b.is_ascii_hexdigit())
}

/// A sintaxe unificada da caixa de busca: `autor:`, `depois:` e `antes:` são filtros
/// estruturados; o resto vira texto livre para o FTS5. Token com prefixo reconhecido mas
/// valor vazio ou data inválida é **ignorado**, não erro — "autor:" sozinho no meio da
/// digitação é um estado passageiro normal, não motivo para a busca inteira falhar.
#[derive(Debug, Default, PartialEq, Eq)]
struct ParsedQuery {
    text: Option<String>,
    author: Option<String>,
    /// Segundos desde a época, inclusive.
    after: Option<i64>,
    /// Segundos desde a época, exclusive — "antes do dia", não "até o dia".
    before: Option<i64>,
}

fn parse_query(raw: &str) -> ParsedQuery {
    let mut parsed = ParsedQuery::default();
    let mut text_words = Vec::new();

    for word in raw.split_whitespace() {
        if let Some(value) = word.strip_prefix("autor:") {
            if !value.is_empty() {
                parsed.author = Some(value.to_owned());
            }
        } else if let Some(value) = word.strip_prefix("depois:") {
            parsed.after = parse_date(value);
        } else if let Some(value) = word.strip_prefix("antes:") {
            parsed.before = parse_date(value);
        } else {
            text_words.push(word);
        }
    }

    if !text_words.is_empty() {
        parsed.text = Some(text_words.join(" "));
    }

    parsed
}

/// `AAAA-MM-DD` → segundos desde a época, à meia-noite UTC daquele dia. `None` para o que não
/// tem essa forma — não vale trazer uma dependência de data só por isto.
fn parse_date(s: &str) -> Option<i64> {
    let mut parts = s.split('-');
    let year: i64 = parts.next()?.parse().ok()?;
    let month: i64 = parts.next()?.parse().ok()?;
    let day: i64 = parts.next()?.parse().ok()?;
    if parts.next().is_some() || !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }

    Some(days_from_civil(year, month, day) * 86_400)
}

/// Dias desde 1970-01-01 para uma data civil (gregoriano proléptico). O algoritmo é de Howard
/// Hinnant (domínio público, amplamente usado em bibliotecas de data de C++); reimplementado
/// aqui porque é a única conta que `parse_date` precisa, e não vale puxar um crate de
/// calendário inteiro para uma multiplicação e algumas divisões inteiras.
fn days_from_civil(year: i64, month: i64, day: i64) -> i64 {
    let y = if month <= 2 { year - 1 } else { year };
    let era = if y >= 0 { y } else { y - 399 } / 400;
    let year_of_era = y - era * 400;
    let month_shifted = if month > 2 { month - 3 } else { month + 9 };
    let day_of_year = (153 * month_shifted + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;

    era * 146_097 + day_of_era - 719_468
}

fn row_to_hit(row: &rusqlite::Row) -> rusqlite::Result<SearchHit> {
    Ok(SearchHit {
        oid: row.get(0)?,
        author: row.get(1)?,
        email: row.get(2)?,
        time: row.get(3)?,
        summary: row.get(4)?,
    })
}

impl Index {
    /// Oid do `HEAD` que este repositório tinha da última vez que foi indexado por completo.
    /// `None` é "nunca indexado" — o job de indexação sempre roda nesse caso.
    pub fn indexed_tip(&self, repo_id: &str) -> Result<Option<String>, IndexError> {
        use rusqlite::OptionalExtension;

        self.with(|conn| {
            conn.query_row(
                "SELECT indexed_tip FROM index_state WHERE repo_id = ?1",
                params![repo_id],
                |row| row.get(0),
            )
            .optional()
        })
        .map_err(IndexError::Read)
    }

    /// Substitui todo o índice de um repositório por `rows` (tabela **e** FTS5) e marca `tip`
    /// como o `HEAD` já coberto — uma transação só, então um crash no meio deixa o índice
    /// **velho** intacto em vez de pela metade. Reindexar de novo do zero é sempre a saída,
    /// então "velho" nunca é pior que "quebrado".
    pub fn replace_commits(
        &self,
        repo_id: &str,
        tip: &str,
        rows: &[CommitRow],
    ) -> Result<(), IndexError> {
        self.with_transaction(|tx| {
            tx.execute("DELETE FROM commits WHERE repo_id = ?1", params![repo_id])?;
            tx.execute(
                "DELETE FROM commits_fts WHERE repo_id = ?1",
                params![repo_id],
            )?;

            {
                let mut insert = tx.prepare(
                    "INSERT INTO commits (repo_id, oid, author, email, ts, summary)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                )?;
                let mut insert_fts = tx.prepare(
                    "INSERT INTO commits_fts (repo_id, oid, author, email, ts, summary)
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                )?;

                for row in rows {
                    insert.execute(params![
                        repo_id,
                        row.oid,
                        row.author,
                        row.email,
                        row.time,
                        row.summary
                    ])?;
                    insert_fts.execute(params![
                        repo_id,
                        row.oid,
                        row.author,
                        row.email,
                        row.time,
                        row.summary
                    ])?;
                }
            }

            tx.execute(
                "INSERT INTO index_state (repo_id, indexed_tip, indexed_at)
                 VALUES (?1, ?2, ?3)
                 ON CONFLICT(repo_id) DO UPDATE SET
                     indexed_tip = excluded.indexed_tip,
                     indexed_at = excluded.indexed_at",
                params![repo_id, tip, now_ms()],
            )?;

            Ok(())
        })
        .map_err(IndexError::Write)
    }

    /// Quantos commits este repositório tem indexados agora. Só para o indicador da UI e para
    /// os testes conferirem que `replace_commits` não deixou duplicata nenhuma.
    pub fn indexed_commit_count(&self, repo_id: &str) -> Result<i64, IndexError> {
        self.with(|conn| {
            conn.query_row(
                "SELECT COUNT(*) FROM commits WHERE repo_id = ?1",
                params![repo_id],
                |row| row.get(0),
            )
        })
        .map_err(IndexError::Read)
    }

    /// Busca por mensagem, autor, hash ou data — a caixa única do Passo 43/44.
    ///
    /// Um token hex de 4-40 caracteres sozinho pula tudo e vira prefixo de oid contra o
    /// índice — "colar um hash curto salta direto para o commit". Senão, a sintaxe unificada
    /// (`autor:`, `depois:`, `antes:`) separa filtros estruturados do texto livre; texto
    /// presente usa o FTS5 (`ORDER BY rank`, mais relevante primeiro), texto ausente consulta
    /// a tabela normal (`ORDER BY ts DESC`, mais recente primeiro — não há relevância para
    /// ordenar quando não há o que combinar).
    ///
    /// `query` vazia devolve lista vazia sem tocar o banco — é o estado de "sem filtro", não
    /// "filtra por nada".
    pub fn search_commits(
        &self,
        repo_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchHit>, IndexError> {
        let trimmed = query.trim();
        if trimmed.is_empty() {
            return Ok(Vec::new());
        }

        if is_hash_like(trimmed) {
            return self.search_by_hash_prefix(repo_id, trimmed, limit);
        }

        let parsed = parse_query(trimmed);
        let author_like = parsed.author.as_ref().map(|value| format!("%{value}%"));

        self.with(|conn| match &parsed.text {
            Some(text) => {
                // `parsed.text` só é `Some` quando havia palavra não reconhecida como filtro,
                // então `build_match_query` nunca vê string vazia aqui.
                let match_query = build_match_query(text).expect("texto não vazio");

                let mut statement = conn.prepare(
                    "SELECT oid, author, email, ts, summary
                     FROM commits_fts
                     WHERE commits_fts MATCH ?1 AND repo_id = ?2
                       AND (?3 IS NULL OR author LIKE ?3)
                       AND (?4 IS NULL OR ts >= ?4)
                       AND (?5 IS NULL OR ts < ?5)
                     ORDER BY rank
                     LIMIT ?6",
                )?;

                let hits: Result<Vec<SearchHit>, rusqlite::Error> = statement
                    .query_map(
                        params![
                            match_query,
                            repo_id,
                            author_like,
                            parsed.after,
                            parsed.before,
                            limit as i64
                        ],
                        row_to_hit,
                    )?
                    .collect();
                hits
            }
            None => {
                let mut statement = conn.prepare(
                    "SELECT oid, author, email, ts, summary
                     FROM commits
                     WHERE repo_id = ?1
                       AND (?2 IS NULL OR author LIKE ?2)
                       AND (?3 IS NULL OR ts >= ?3)
                       AND (?4 IS NULL OR ts < ?4)
                     ORDER BY ts DESC
                     LIMIT ?5",
                )?;

                let hits: Result<Vec<SearchHit>, rusqlite::Error> = statement
                    .query_map(
                        params![
                            repo_id,
                            author_like,
                            parsed.after,
                            parsed.before,
                            limit as i64
                        ],
                        row_to_hit,
                    )?
                    .collect();
                hits
            }
        })
        .map_err(IndexError::Read)
    }

    /// Prefixo de oid contra a tabela normal — "com índice" quer dizer a chave primária
    /// `(repo_id, oid)`, não o FTS5. Hash não é texto para full-text search: é um valor exato
    /// (até onde foi digitado), e um índice de prefixo comum já é o jeito certo de achá-lo.
    fn search_by_hash_prefix(
        &self,
        repo_id: &str,
        prefix: &str,
        limit: usize,
    ) -> Result<Vec<SearchHit>, IndexError> {
        let pattern = format!("{}%", prefix.to_ascii_lowercase());

        self.with(|conn| {
            let mut statement = conn.prepare(
                "SELECT oid, author, email, ts, summary
                 FROM commits
                 WHERE repo_id = ?1 AND oid LIKE ?2
                 ORDER BY ts DESC
                 LIMIT ?3",
            )?;

            let hits: Result<Vec<SearchHit>, rusqlite::Error> = statement
                .query_map(params![repo_id, pattern, limit as i64], row_to_hit)?
                .collect();
            hits
        })
        .map_err(IndexError::Read)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(oid: &str, author: &str, summary: &str) -> CommitRow {
        CommitRow {
            oid: oid.to_owned(),
            author: author.to_owned(),
            email: format!("{author}@example.com"),
            time: 1_700_000_000,
            summary: summary.to_owned(),
        }
    }

    #[test]
    fn nunca_indexado_e_none() {
        let index = Index::in_memory();
        assert_eq!(index.indexed_tip("repo").unwrap(), None);
    }

    #[test]
    fn replace_substitui_em_vez_de_acumular() {
        let index = Index::in_memory();

        index
            .replace_commits(
                "repo",
                "tip1",
                &[row("a", "Ana", "primeiro"), row("b", "Ana", "segundo")],
            )
            .unwrap();
        assert_eq!(index.indexed_commit_count("repo").unwrap(), 2);
        assert_eq!(index.indexed_tip("repo").unwrap().as_deref(), Some("tip1"));

        // Reindexado com um conjunto menor: as duas linhas antigas não podem sobrar.
        index
            .replace_commits("repo", "tip2", &[row("c", "Ana", "terceiro")])
            .unwrap();
        assert_eq!(index.indexed_commit_count("repo").unwrap(), 1);
        assert_eq!(index.indexed_tip("repo").unwrap().as_deref(), Some("tip2"));
    }

    #[test]
    fn repositorios_diferentes_nao_se_misturam() {
        let index = Index::in_memory();

        index
            .replace_commits("a", "tip-a", &[row("1", "Ana", "x")])
            .unwrap();
        index
            .replace_commits("b", "tip-b", &[row("1", "Bia", "x"), row("2", "Bia", "y")])
            .unwrap();

        assert_eq!(index.indexed_commit_count("a").unwrap(), 1);
        assert_eq!(index.indexed_commit_count("b").unwrap(), 2);
    }

    #[test]
    fn busca_encontra_por_mensagem_e_por_autor() {
        let index = Index::in_memory();
        index
            .replace_commits(
                "repo",
                "tip",
                &[
                    row("a", "Ana Paula", "corrige o parser de datas"),
                    row("b", "Beto", "adiciona testes do parser"),
                    row("c", "Beto", "atualiza dependências"),
                ],
            )
            .unwrap();

        let by_message = index.search_commits("repo", "parser", 10).unwrap();
        assert_eq!(by_message.len(), 2);
        assert!(by_message.iter().all(|hit| hit.summary.contains("parser")));

        let by_author = index.search_commits("repo", "Beto", 10).unwrap();
        assert_eq!(by_author.len(), 2);
        assert!(by_author.iter().all(|hit| hit.author == "Beto"));

        assert!(index
            .search_commits("repo", "inexistente", 10)
            .unwrap()
            .is_empty());
    }

    #[test]
    fn busca_e_prefixo_e_incremental() {
        let index = Index::in_memory();
        index
            .replace_commits(
                "repo",
                "tip",
                &[row("a", "Ana", "corrige o parser de datas")],
            )
            .unwrap();

        // "pars" tem que achar "parser" mesmo sem a palavra inteira — é o que faz filtrar a
        // cada tecla, sem esperar a palavra terminar.
        for prefix in ["p", "pa", "par", "pars", "parse", "parser"] {
            assert_eq!(
                index.search_commits("repo", prefix, 10).unwrap().len(),
                1,
                "prefixo {prefix:?} deveria achar o commit"
            );
        }
    }

    #[test]
    fn busca_nao_quebra_com_sintaxe_de_operador_fts5() {
        let index = Index::in_memory();
        index
            .replace_commits("repo", "tip", &[row("a", "Ana", "fix AND cleanup")])
            .unwrap();

        // Sem a proteção de aspas, `AND`, `-erro` ou uma aspa solta seriam sintaxe de operador
        // do FTS5 e a consulta falharia — aqui têm que ser tratados como texto literal.
        for query in ["AND", "-erro", "\"", "OR NOT"] {
            assert!(
                index.search_commits("repo", query, 10).is_ok(),
                "consulta {query:?} não deveria falhar"
            );
        }
    }

    #[test]
    fn busca_vazia_nao_toca_o_banco() {
        let index = Index::in_memory();
        index
            .replace_commits("repo", "tip", &[row("a", "Ana", "x")])
            .unwrap();

        assert!(index.search_commits("repo", "", 10).unwrap().is_empty());
        assert!(index.search_commits("repo", "   ", 10).unwrap().is_empty());
    }

    #[test]
    fn hash_curto_ou_completo_salta_direto_para_o_commit() {
        let index = Index::in_memory();
        index
            .replace_commits(
                "repo",
                "tip",
                &[
                    row(
                        "abcd123def456abc123def456abc123def456abc",
                        "Ana",
                        "primeiro",
                    ),
                    row(
                        "abcd999def456",
                        "Ana",
                        "outro, prefixo parecido mas diferente",
                    ),
                ],
            )
            .unwrap();

        // Curto: acha pelo prefixo.
        let short = index.search_commits("repo", "abcd123", 10).unwrap();
        assert_eq!(short.len(), 1);
        assert_eq!(short[0].oid, "abcd123def456abc123def456abc123def456abc");

        // Completo: acha exatamente um.
        let full = index
            .search_commits("repo", "abcd123def456abc123def456abc123def456abc", 10)
            .unwrap();
        assert_eq!(full.len(), 1);

        // Maiúsculo: git é case-insensitive para hash colado.
        let upper = index.search_commits("repo", "ABCD123", 10).unwrap();
        assert_eq!(upper.len(), 1);

        // Prefixo curto o bastante para casar os dois (ainda ≥ 4, o mínimo de `is_hash_like`).
        let ambiguous = index.search_commits("repo", "abcd", 10).unwrap();
        assert_eq!(ambiguous.len(), 2);
    }

    #[test]
    fn sintaxe_unificada_combina_autor_e_intervalo_de_datas() {
        let index = Index::in_memory();
        let mut rows = vec![
            row("a", "Ana", "corrige bug de datas"),
            row("b", "Beto", "corrige bug de rede"),
        ];
        rows[0].time = parse_date("2024-06-10").unwrap();
        rows[1].time = parse_date("2024-06-10").unwrap();
        index.replace_commits("repo", "tip", &rows).unwrap();

        let by_author_and_text = index.search_commits("repo", "autor:Ana bug", 10).unwrap();
        assert_eq!(by_author_and_text.len(), 1);
        assert_eq!(by_author_and_text[0].oid, "a");

        // Só filtro estruturado, sem texto livre nenhum: vai pela tabela normal, não FTS5.
        let by_author_only = index.search_commits("repo", "autor:Beto", 10).unwrap();
        assert_eq!(by_author_only.len(), 1);
        assert_eq!(by_author_only[0].oid, "b");

        let in_range = index
            .search_commits("repo", "depois:2024-06-01 antes:2024-06-20", 10)
            .unwrap();
        assert_eq!(in_range.len(), 2);

        let out_of_range = index
            .search_commits("repo", "depois:2024-07-01", 10)
            .unwrap();
        assert!(out_of_range.is_empty());
    }

    #[test]
    fn autor_vazio_ou_data_invalida_e_ignorado_sem_erro() {
        let index = Index::in_memory();
        index
            .replace_commits("repo", "tip", &[row("a", "Ana", "corrige bug")])
            .unwrap();

        // "autor:" sem valor e "depois:" com data que não existe: nenhum dos dois deveria
        // filtrar nada, só o texto livre continua valendo.
        let hits = index
            .search_commits("repo", "autor: depois:nao-e-data bug", 10)
            .unwrap();
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn dias_desde_epoca_bate_com_datas_conhecidas() {
        assert_eq!(parse_date("1970-01-01"), Some(0));
        assert_eq!(parse_date("1970-01-02"), Some(86_400));
        assert_eq!(parse_date("1969-12-31"), Some(-86_400));
        assert_eq!(parse_date("2000-03-01"), Some(951_868_800));
        assert_eq!(parse_date("lixo"), None);
        assert_eq!(parse_date("2024-13-01"), None);
    }

    #[test]
    fn busca_em_cinquenta_mil_commits_e_rapida() {
        // O aceite do Passo 43 é "digitar filtra em < 20ms medidos no servidor". 50k linhas é
        // maior que qualquer coisa que o Passo 42 mediu em minutos deste projeto, e ainda
        // assim a consulta é uma busca em índice, não uma varredura.
        let index = Index::in_memory();
        let rows: Vec<CommitRow> = (0..50_000)
            .map(|i| {
                row(
                    &format!("oid{i}"),
                    "Ana",
                    &format!("commit número {i} sobre o parser"),
                )
            })
            .collect();
        index.replace_commits("repo", "tip", &rows).unwrap();

        let start = std::time::Instant::now();
        let hits = index.search_commits("repo", "parser", 500).unwrap();
        let elapsed = start.elapsed();

        assert_eq!(hits.len(), 500, "limit corta em 500, não devolve os 50k");
        assert!(
            elapsed.as_millis() < 100,
            "busca em 50k commits levou {elapsed:?} — bem acima do orçamento do Passo 43"
        );
    }
}
