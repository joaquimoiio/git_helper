//! Domínio git: leitura via `git2` atrás do trait `RepoRead`, execução via shell-out para
//! o `git` do sistema, modelos e parsers. Não conhece HTTP nem axum.

pub mod discover;
pub mod exec;
pub mod model;
pub mod parse;
pub mod patch;
pub mod read;
