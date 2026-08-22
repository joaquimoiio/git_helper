//! Parsers da saída do `git`.
//!
//! Só formatos que o git promete manter: `-z`, `--format` com `%x00`, e o `--progress`, que é
//! saída para humano mas cuja gramática não muda há mais de uma década. Porcelana instável
//! (`git status` sem `--porcelain`, `git log` no formato default) não entra aqui.

pub mod progress;
pub mod records;
pub mod status_v2;
