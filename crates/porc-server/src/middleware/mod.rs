//! Camadas do router, na ordem em que o request as atravessa:
//! `trace → origin_guard → rate_limit → auth → csrf → rotas`.
//!
//! A ordem não é estética: o guard de origem precisa recusar o request **antes** de
//! qualquer coisa olhar cookie, e o CSRF só faz sentido depois de a sessão ser válida.

pub mod auth;
pub mod csrf;
pub mod origin;
