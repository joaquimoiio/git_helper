//! Constrói o frontend quando ele vai ser embutido.
//!
//! O objetivo do bloco é `cargo build --release` produzir **um arquivo**, sem passo manual
//! antes. Quem garante isso é este script: ele roda o build do Vite e deixa `web/dist`
//! pronto para o `rust-embed` ler em tempo de compilação.
//!
//! Em debug com `dev-proxy` não roda nada — ali quem serve o frontend é o próprio Vite, e
//! esperar por um build de produção a cada `cargo run` mataria o ciclo.

use std::{
    env,
    path::{Path, PathBuf},
    process::Command,
};

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("cargo define isto"));
    let web = manifest.join("../../web");

    // O conteúdo de `dist` é derivado; o que dispara rebuild é a fonte. `assets/` entra
    // porque as fontes vão para dentro do bundle pelo `url()` do CSS.
    for watched in [
        web.join("src"),
        web.join("index.html"),
        web.join("package.json"),
        web.join("package-lock.json"),
        web.join("vite.config.ts"),
        manifest.join("../../assets"),
    ] {
        println!("cargo:rerun-if-changed={}", watched.display());
    }

    let embed = env::var_os("CARGO_FEATURE_EMBED_WEB").is_some();
    let dev_proxy = env::var_os("CARGO_FEATURE_DEV_PROXY").is_some();
    let debug = env::var("PROFILE").as_deref() == Ok("debug");

    // Espelha exatamente o `cfg` do `lib.rs`: só constrói o que vai ser lido.
    if !embed || (dev_proxy && debug) {
        return;
    }

    // `npm ci` só quando não há `node_modules`: ele apaga e reinstala tudo, e pagar isso a
    // cada `cargo build --release` seriam minutos por build. Quando roda, roda pelo
    // lockfile, que é o ponto de usar `ci` em vez de `install`.
    if !web.join("node_modules").exists() {
        run(&web, "ci", &["ci"]);
    }
    run(&web, "run build", &["run", "build"]);

    let index = web.join("dist/index.html");
    assert!(
        index.exists(),
        "o build do Vite terminou sem gerar {} — nada para embutir",
        index.display()
    );
}

fn run(dir: &Path, label: &str, args: &[&str]) {
    // No Windows `npm` é um script, não um executável: sem o `.cmd` o `CreateProcess` falha.
    let npm = if cfg!(windows) { "npm.cmd" } else { "npm" };

    let status = Command::new(npm)
        .args(args)
        .current_dir(dir)
        .status()
        .unwrap_or_else(|err| panic!("não consegui executar `{npm} {label}`: {err}"));

    assert!(status.success(), "`{npm} {label}` falhou: {status}");
}
