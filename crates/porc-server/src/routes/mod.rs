//! Handlers HTTP. Cada módulo é um recurso da API (`/api/v1/<recurso>/…`).
//!
//! Nenhum deles fala com libgit2 direto — quem sabe de git é o `porc-git`.

pub mod fs;
pub mod jobs;
pub mod recents;
pub mod repos;
