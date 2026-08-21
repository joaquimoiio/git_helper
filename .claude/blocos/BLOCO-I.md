# BLOCO I — empacotar

**Objetivo:** um binário por sistema, lançador `.desktop` no Linux, e CI gerando tudo.

**Ao fim do bloco:** um clique no lançador sobe o serviço e abre a página.

---

## Detalhe técnico do bloco

**O `.desktop`.** O lançador chama `porc` direto. A lógica de "não subir uma segunda
instância" já está no Passo 10 — o `.desktop` não precisa saber de nada disso, só
executar o binário. Instalação de usuário (sem root):

```
~/.local/bin/porc
~/.local/share/applications/porcelain.desktop
~/.local/share/icons/hicolor/256x256/apps/porcelain.png
```
Depois de instalar: `update-desktop-database ~/.local/share/applications`.

**Binário Linux.** Alvo `x86_64-unknown-linux-gnu` com glibc antiga o suficiente, ou
`musl` se `rusqlite bundled` + `libgit2-sys` cooperarem (ambos compilam C — validar cedo).
Verificar com `ldd` que as dependências dinâmicas são mínimas. `git`, SSH agent e o
credential helper do usuário continuam vindo do sistema — isso é por design.

**Perfil de release:** `lto = "fat"`, `codegen-units = 1`, `strip = true`,
`opt-level = 3`, `panic = "abort"` (avaliar contra a necessidade de capturar panic em job).

**Metas a medir de verdade neste bloco**, não a estimar: boot < 1s, primeira página do
log < 50ms, busca indexada < 20ms, primeiros hits de pickaxe < 150ms.

---

## Passos

### Passo 97 — perfil de release e medição de boot
Perfil otimizado + medição instrumentada do tempo de boot até servir, impressa em modo
verboso.
**Aceite:** `cargo build --release` e boot medido abaixo de 1s.

### Passo 98 — build Linux
Alvo Linux com deps mínimas, verificação de `ldd`, e checagem no boot de que `git` ≥ 2.30
existe no PATH com erro legível se não existir.
**Aceite:** o binário roda numa máquina Linux limpa (ou container) sem instalar nada além
de `git`.

### Passo 99 — ícone e `.desktop`
Ícone monocromático (SVG + PNG 256px) e `packaging/porcelain.desktop` com `Name`,
`Comment`, `Exec`, `Icon`, `Terminal=false`, `Categories=Development;RevisionControl;`.
**Aceite:** o lançador aparece no menu do sistema com o ícone certo.

### Passo 100 — script de instalação
`packaging/install.sh` copiando binário, `.desktop` e ícone para os caminhos de usuário e
rodando `update-desktop-database`. E um `uninstall.sh`.
**Aceite:** instalar, clicar no lançador, o navegador abrir no app.

### Passo 101 — build macOS
`aarch64-apple-darwin` e `x86_64-apple-darwin`, com `open` funcionando e os caminhos de
config/dados corretos (`~/Library/…`).
**Aceite:** binário macOS roda e abre o navegador.

### Passo 102 — smoke test end-to-end
Script gerando um repo sintético grande (100k commits, muitos merges) e uma suíte que
exercita boot, log, busca e as metas de performance.
**Aceite:** a suíte passa e imprime os números medidos de cada meta.

### Passo 103 — CI
GitHub Actions: build Linux + macOS (arm64 e x86_64), `clippy`, testes, e anexo dos
binários à release por tag.
**Aceite:** uma tag produz uma release com os binários dos dois sistemas.

---

**Fim do bloco:** atualizar `PROGRESSO.md`. **v1 completo.**
Próximo horizonte (v2, não iniciar sem pedido): submódulos, bisect, séries de patch,
sparse checkout, Git LFS, hooks manager, export de árvore.
