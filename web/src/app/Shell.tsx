/* Casca de três painéis: sidebar → centro → detalhe.
 *
 * Hierarquia por valor de superfície e por borda de 1px, nunca por sombra. As barras de
 * topo e de status são fixas; só a faixa do meio rola, e cada painel rola por conta
 * própria (`min-h-0` + `overflow-auto` em cada um, senão o grid estica e a página inteira
 * ganha uma barra de rolagem).
 *
 * O conteúdo é placeholder — o Bloco C traz o repositório de verdade.
 */

import { useEffect } from "react";

import { useLayout } from "../lib/layout";
import { useDocumentTitle } from "../lib/title";
import { Splitter } from "./Splitter";
import { ThemeSwitch } from "./ThemeSwitch";

/* Placeholder, do mesmo jeito que a lista de commits abaixo. O Bloco C troca isto pelo
 * repositório de verdade, e a aba passa a segui-lo sozinha. */
const REPO = "git_helper";
const BRANCH = "main";

function TopBar() {
  return (
    <header className="flex items-center justify-between gap-4 h-8 px-3 border-b border-n-5 bg-n-1 shrink-0">
      <div className="flex items-baseline gap-2 min-w-0">
        <span className="text-base text-n-11 truncate">{REPO}</span>
        <span className="text-n-7">/</span>
        <span className="text-sm font-mono text-n-9 truncate">{BRANCH}</span>
      </div>
      <ThemeSwitch />
    </header>
  );
}

function StatusBar() {
  return (
    <footer className="flex items-center justify-between gap-4 h-6 px-3 border-t border-n-5 bg-n-1 text-xs text-n-8 shrink-0">
      <span className="font-mono">ctrl+b sidebar · ctrl+d detalhe</span>
      <span className="font-mono">porcelain 0.1.0</span>
    </footer>
  );
}

function SidebarGroup({ title, items }: { title: string; items: readonly string[] }) {
  return (
    <section className="py-2">
      <h2 className="px-3 pb-1 text-xs uppercase tracking-[0.1em] text-n-8">{title}</h2>
      <ul>
        {items.map((item) => (
          <li
            key={item}
            className="px-3 py-0.5 text-sm text-n-10 truncate hover:bg-n-3 transition-colors duration-(--duration-fast)"
          >
            {item}
          </li>
        ))}
      </ul>
    </section>
  );
}

function Sidebar({ width }: { width: number }) {
  return (
    <aside className="shrink-0 overflow-y-auto border-r border-n-5 bg-n-1" style={{ width }}>
      <SidebarGroup title="branches" items={["main", "feat/log-canvas", "fix/status-z"]} />
      <SidebarGroup title="remotes" items={["origin/main", "origin/feat/log-canvas"]} />
      <SidebarGroup title="tags" items={["v0.1.0"]} />
      <SidebarGroup title="stashes" items={["stash@{0}"]} />
    </aside>
  );
}

const COMMITS = [
  ["2cea64a", "andaime do projeto: contexto, progresso, blocos e comandos", "há 2h"],
  ["9f1c40b", "servidor: shutdown gracioso por rota e por sinal", "há 3h"],
  ["4b77e12", "csrf: double-submit em todo método que muda estado", "há 5h"],
  ["a03de55", "origin: recusar host fora do loopback", "ontem"],
  ["7cc9081", "sessão: trocar token de boot por cookie HttpOnly", "ontem"],
] as const;

function Center() {
  return (
    <main className="flex-1 min-w-0 overflow-y-auto bg-n-0">
      <ul>
        {COMMITS.map(([hash, message, when], index) => (
          <li
            key={hash}
            className={`flex items-baseline gap-3 px-3 py-1 border-b border-n-5 transition-colors duration-(--duration-fast) ${
              index === 0 ? "bg-n-4" : "hover:bg-n-3"
            }`}
          >
            <span className="font-mono text-sm text-n-9 shrink-0">{hash}</span>
            <span className="text-base text-n-11 truncate">{message}</span>
            <span className="ml-auto font-mono text-xs text-n-8 shrink-0">{when}</span>
          </li>
        ))}
      </ul>
    </main>
  );
}

const DIFF = [
  [" ", 'pub fn router(state: AppState) -> Router {'],
  ["-", '    Router::new().route("/", get(page::index))'],
  ["+", '    Router::new().route("/", get(shell::index))'],
  [" ", '        .route("/health", get(health))'],
] as const;

function Detail({ width }: { width: number }) {
  return (
    <aside className="shrink-0 overflow-y-auto border-l border-n-5 bg-n-1" style={{ width }}>
      <div className="px-3 py-2 border-b border-n-5">
        <p className="text-base text-n-11">
          andaime do projeto: contexto, progresso, blocos e comandos
        </p>
        <p className="mt-1 text-sm font-mono text-n-9">2cea64a · joaquimoiio · há 2h</p>
      </div>
      <div className="px-3 py-2 border-b border-n-5 text-sm font-mono text-n-9">
        crates/porc-server/src/lib.rs
      </div>
      <pre className="text-sm font-mono leading-5 py-1">
        {DIFF.map(([sign, line], index) => (
          <div
            key={index}
            className="px-3 whitespace-pre"
            style={
              sign === "+"
                ? { background: "var(--add-bg)", color: "var(--add-fg)" }
                : sign === "-"
                  ? { background: "var(--del-bg)", color: "var(--del-fg)" }
                  : undefined
            }
          >
            {sign}
            {line}
          </div>
        ))}
      </pre>
    </aside>
  );
}

/* Ctrl+B e Ctrl+D. Ficam fora da lista que o navegador rouba (Ctrl+W/T/N/L e
 * Ctrl+Shift+*), mas Ctrl+D é "favoritar" — daí o `preventDefault`. Aceita `metaKey`
 * porque no macOS o dedo do usuário vai para Cmd. */
function usePanelShortcuts() {
  const toggle = useLayout((state) => state.toggle);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (!(event.ctrlKey || event.metaKey) || event.shiftKey || event.altKey) return;

      const panel = event.key === "b" ? "sidebar" : event.key === "d" ? "detail" : null;
      if (!panel) return;

      event.preventDefault();
      toggle(panel);
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [toggle]);
}

export function Shell() {
  const sidebarWidth = useLayout((state) => state.sidebarWidth);
  const detailWidth = useLayout((state) => state.detailWidth);
  const sidebarCollapsed = useLayout((state) => state.sidebarCollapsed);
  const detailCollapsed = useLayout((state) => state.detailCollapsed);

  useDocumentTitle(REPO, BRANCH);
  usePanelShortcuts();

  return (
    <div className="h-full flex flex-col bg-n-0 text-n-11">
      <TopBar />
      <div className="flex-1 min-h-0 flex">
        {!sidebarCollapsed && (
          <>
            <Sidebar width={sidebarWidth} />
            <Splitter panel="sidebar" edge="end" />
          </>
        )}
        <Center />
        {!detailCollapsed && (
          <>
            <Splitter panel="detail" edge="start" />
            <Detail width={detailWidth} />
          </>
        )}
      </div>
      <StatusBar />
    </div>
  );
}
