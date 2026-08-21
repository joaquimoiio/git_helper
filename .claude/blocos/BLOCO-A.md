# BLOCO A — servidor de pé

**Objetivo:** um binário `porc` que sobe um servidor em `127.0.0.1`, fechado por padrão,
abre o navegador sozinho, recusa segunda instância e encerra limpo.

**Ao fim do bloco:** `porc` abre uma aba autenticada mostrando um "hello" do servidor;
rodar `porc` de novo só abre outra aba; `Ctrl+C` e o botão de encerrar funcionam.

---

## Detalhe técnico do bloco

**Crates iniciais:** `axum`, `tokio` (rt-multi-thread, macros, signal), `tower`,
`tower-http` (trace, set-header), `tracing`, `tracing-subscriber`, `serde`, `serde_json`,
`clap` (derive), `getrandom`, `subtle`, `directories`, `anyhow`, `thiserror`.

**Ordem das camadas no `Router` (fixa):**
`trace → origin_guard → rate_limit → auth → csrf → rotas`

**Por que o guard de Host/Origin é a defesa principal:** bind em `127.0.0.1` não protege
contra DNS rebinding. Um site aberto noutra aba resolve seu domínio para `127.0.0.1`; o
browser da vítima considera o request válido do ponto de vista de rede e o entrega ao
servidor. O que denuncia o ataque é o header `Host` carregar o domínio do atacante, não
`127.0.0.1`. Por isso o guard vem antes de tudo e vale também no upgrade do WebSocket
(WS não sofre CORS e escapa de `SameSite`).

**Lockfile:** `~/.local/share/porcelain/porc.lock` (macOS: `~/Library/Application Support/...`),
contendo `{ pid, port, token_hint }`. Segunda execução lê o arquivo, confere se o PID
está vivo e se a porta responde `/health` com a assinatura esperada; se sim, só abre a
aba. Lockfile órfão (processo morto) é sobrescrito, não trava o boot.

---

## Passos

### Passo 1 — toolchain e workspace vazio
Instalar Rust via `rustup`. Criar `Cargo.toml` de workspace com os quatro membros
(`porc-cli`, `porc-server`, `porc-git`, `porc-index`), cada um só com um `lib.rs`/`main.rs`
mínimo. `porc-cli` é o único binário.
**Aceite:** `cargo build` compila sem warning.

### Passo 2 — versão e esqueleto de crates
`porc-cli` imprime nome e versão. `porc-server` expõe uma função pública `serve()` ainda
vazia. Confirma que a fronteira de crates está montada.
**Aceite:** `cargo run -- --version` imprime `porcelain 0.1.0`.

### Passo 3 — axum servindo `/health`
`porc-server` sobe axum em `127.0.0.1` numa porta fixa (7867) com `GET /health`
devolvendo JSON `{ name, version, pid }`. `tracing-subscriber` configurado.
**Aceite:** `curl http://127.0.0.1:7867/health` responde o JSON.

### Passo 4 — porta dinâmica e URL real
Tentar a porta preferida; se ocupada, cair para porta efêmera (`:0`) e descobrir a porta
real pelo listener. Imprimir a URL no stdout.
**Aceite:** subir duas vezes com a porta ocupada e ver a segunda escolher outra porta.

### Passo 5 — token de sessão e troca por cookie
Gerar 256 bits com `getrandom` a cada boot, nunca persistir. `POST /api/v1/session`
recebe `{ token }`, compara em tempo constante (`subtle`), responde
`Set-Cookie: porc_sess=…; HttpOnly; SameSite=Strict; Path=/`.
**Aceite:** `curl` com o token correto recebe o cookie; com token errado recebe 401.

### Passo 6 — middleware de auth
`middleware/auth.rs` protegendo tudo exceto `/health` e a rota de handshake. Sem cookie
válido → 401.
**Aceite:** `curl /api/v1/whoami` sem cookie dá 401; com cookie dá 200.

### Passo 7 — guard de Host/Origin
`middleware/origin.rs`. 403 se `Host` não for `127.0.0.1:<porta>` ou `localhost:<porta>`.
Se `Origin` presente, tem que bater; `Origin: null` é rejeitado.
**Aceite:** `curl -H "Host: evil.com" …` dá 403 mesmo com cookie válido. Este teste é o
mais importante do bloco.

### Passo 8 — CSRF double-submit
`middleware/csrf.rs`. O handshake também seta cookie `porc_csrf` legível por JS. Todo
método ≠ GET/HEAD exige header `X-CSRF-Token` batendo com o cookie.
**Aceite:** `POST` sem o header dá 403; com o header dá 200.

### Passo 9 — abrir o navegador
`porc-cli/browser.rs`: `xdg-open` no Linux, `open` no macOS, `cmd /c start` no Windows.
Abre a URL já com `?t=<token>`. Falha em abrir não derruba o servidor — só imprime a URL.
**Aceite:** rodar `porc` abre a aba sozinho.

### Passo 10 — instância única (lockfile)
Escrever/ler o lockfile com PID e porta. Segunda execução detecta a instância viva, abre
só uma aba nova e sai com código 0. Lockfile órfão é sobrescrito.
**Aceite:** rodar `porc` duas vezes → um só processo, duas abas.

### Passo 11 — flags de linha de comando
`clap`: `--port <n>`, `--no-browser`, `--repo <caminho>`, `--enable-terminal`.
`--repo` só valida e guarda o caminho por enquanto.
**Aceite:** `porc --help` mostra as quatro; `porc --no-browser` não abre aba.

### Passo 12 — encerrar limpo
`POST /api/v1/shutdown` dispara shutdown gracioso do axum. `Ctrl+C` (`tokio::signal`)
faz o mesmo caminho. Ambos removem o lockfile. Botão provisório na página `/`.
**Aceite:** `curl -X POST …/shutdown` com cookie+CSRF encerra o processo e o lockfile
some. `Ctrl+C` idem.

---

**Fim do bloco:** atualizar `PROGRESSO.md` e dizer
*"bloco concluído, pode dar `/clear` e rodar `/bloco B`"*.
