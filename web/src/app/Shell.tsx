/* Casca de três painéis: sidebar → centro → detalhe.
 *
 * Hierarquia por valor de superfície e por borda de 1px, nunca por sombra. As barras de
 * topo e de status são fixas; só a faixa do meio rola, e cada painel rola por conta
 * própria (`min-h-0` + `overflow-auto` em cada um, senão o grid estica e a página inteira
 * ganha uma barra de rolagem).
 *
 * Sem repositório aberto, o centro é o navegador de pastas. Com um repositório aberto, o
 * centro passa a ser o log — que chega no Bloco D; por ora ele diz o que sabe do `HEAD`.
 */

import { useCallback, useEffect, useState } from "react";

import { CloneRepo } from "../features/browse/CloneRepo";
import { Discovered } from "../features/browse/Discovered";
import { FolderBrowser } from "../features/browse/FolderBrowser";
import { NewRepo } from "../features/browse/NewRepo";
import { Recents } from "../features/browse/Recents";
import { AskpassPrompt } from "../features/jobs/AskpassPrompt";
import { JobsPanel } from "../features/jobs/JobsPanel";
import type { Repo } from "../lib/api-types";
import { useJobEvents, useStartTestJob } from "../lib/jobs";
import { useLayout } from "../lib/layout";
import { useOpenRepo, useOpenRepoMutation, useRepoSelection } from "../lib/repo";
import { useDocumentTitle } from "../lib/title";
import { Splitter } from "./Splitter";
import { ThemeSwitch } from "./ThemeSwitch";

function TopBar({ repo }: { repo: Repo | undefined }) {
  const setRepoId = useRepoSelection((state) => state.setRepoId);

  return (
    <header className="flex items-center justify-between gap-4 h-8 px-3 border-b border-n-5 bg-n-1 shrink-0">
      <div className="flex items-baseline gap-2 min-w-0">
        {repo ? (
          <>
            <span className="text-base text-n-11 truncate">{repo.name}</span>
            <span className="text-n-7">/</span>
            <span className="text-sm font-mono text-n-9 truncate">{repo.branch}</span>
            {repo.detached && <span className="text-xs text-n-8">detached</span>}
            {repo.bare && <span className="text-xs text-n-8">bare</span>}
            {repo.head.kind === "unborn" && <span className="text-xs text-n-8">sem commits</span>}
            <button
              type="button"
              onClick={() => setRepoId(null)}
              className="ml-2 text-xs text-n-9 hover:text-n-11 transition-colors duration-(--duration-fast)"
            >
              trocar
            </button>
          </>
        ) : (
          <span className="text-base text-n-9">nenhum repositório aberto</span>
        )}
      </div>
      <ThemeSwitch />
    </header>
  );
}

function StatusBar() {
  const test = useStartTestJob();

  return (
    <footer className="flex items-center justify-between gap-4 h-6 px-3 border-t border-n-5 bg-n-1 text-xs text-n-8 shrink-0">
      <span className="font-mono">ctrl+b sidebar · ctrl+d detalhe</span>
      <div className="flex items-center gap-3">
        {/* Prova o canal de jobs de ponta a ponta sem depender de rede. Sai quando o clone
            chegar (Passo 31) e houver um job de verdade para exercitar a mesma infra. */}
        <button
          type="button"
          onClick={() => test.mutate()}
          disabled={test.isPending}
          className="font-mono hover:text-n-11 disabled:text-n-7 transition-colors duration-(--duration-fast)"
        >
          job de teste
        </button>
        <span className="font-mono">porcelain 0.1.0</span>
      </div>
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

function Sidebar({ width, repo }: { width: number; repo: Repo | undefined }) {
  return (
    <aside className="shrink-0 overflow-y-auto border-r border-n-5 bg-n-1" style={{ width }}>
      {repo ? (
        <>
          <section className="px-3 py-2 border-b border-n-5">
            <p className="text-base text-n-11 truncate">{repo.name}</p>
            <p className="mt-0.5 text-xs font-mono text-n-8 break-all">{repo.path}</p>
          </section>
          {/* Placeholder até o Bloco F: refs de verdade pedem rota própria. */}
          <SidebarGroup title="branches" items={[repo.branch]} />
        </>
      ) : (
        <p className="px-3 py-2 text-sm text-n-9">
          escolha uma pasta no centro. as marcadas com ◆ já são repositórios.
        </p>
      )}
    </aside>
  );
}

function Center({ repo }: { repo: Repo | undefined }) {
  const open = useOpenRepoMutation();
  // A pasta em que o navegador está. Sobe até aqui porque é o formulário de criar, que é irmão
  // do navegador e não filho dele, que precisa saber onde o `git init` vai acontecer.
  const [browsing, setBrowsing] = useState<string | null>(null);
  // Um formulário de cada vez: `null`, "novo" ou "clone". Dois abertos ao mesmo tempo
  // competiriam pelo mesmo pedaço da tela e pelo mesmo Enter.
  const [form, setForm] = useState<"new" | "clone" | null>(null);

  // Estáveis: sem isto o `useEffect` que reporta o caminho no `FolderBrowser` roda a cada
  // render do `Center`.
  const startNew = useCallback(() => setForm("new"), []);
  const startClone = useCallback(() => setForm("clone"), []);
  const closeForm = useCallback(() => setForm(null), []);

  if (repo) {
    return (
      <main className="flex-1 min-w-0 min-h-0 overflow-y-auto bg-n-0">
        <div className="px-3 py-2 border-b border-n-5">
          <p className="text-base text-n-11">{repo.name}</p>
          <p className="mt-0.5 text-sm font-mono text-n-9">
            {repo.head.kind === "unborn"
              ? `${repo.branch} · nenhum commit ainda`
              : `${repo.branch} · ${repo.head.commit.slice(0, 12)}`}
          </p>
        </div>
        <p className="px-3 py-2 text-sm text-n-8">o log chega no Bloco D.</p>
      </main>
    );
  }

  return (
    <main className="flex-1 min-w-0 min-h-0 flex flex-col bg-n-0">
      {open.isError && (
        <p className="px-3 py-1 text-sm text-error bg-error-bg shrink-0">
          {(open.error as Error).message}
        </p>
      )}
      <Recents onPick={(entry) => open.mutate(entry.path)} />
      <Discovered onPick={(entry) => open.mutate(entry.path)} />
      <div className="flex-1 min-h-0">
        <FolderBrowser
          onPick={(entry) => open.mutate(entry.path)}
          onPathChange={setBrowsing}
          onNew={startNew}
          onClone={startClone}
        />
      </div>
      <NewRepo parent={browsing} open={form === "new"} onClose={closeForm} />
      <CloneRepo parent={browsing} open={form === "clone"} onClose={closeForm} />
    </main>
  );
}

function Detail({ width, repo }: { width: number; repo: Repo | undefined }) {
  return (
    <aside className="shrink-0 overflow-y-auto border-l border-n-5 bg-n-1" style={{ width }}>
      {repo ? (
        <dl className="text-sm">
          {(
            [
              ["caminho", repo.path],
              ["head", repo.head.kind],
              ["branch", repo.branch],
              ["commit", repo.head.kind === "unborn" ? "—" : repo.head.commit],
              ["bare", repo.bare ? "sim" : "não"],
              ["repo id", repo.repoId],
            ] as const
          ).map(([label, value]) => (
            <div key={label} className="px-3 py-1 border-b border-n-5">
              <dt className="text-xs uppercase tracking-[0.1em] text-n-8">{label}</dt>
              <dd className="font-mono text-n-10 break-all">{value}</dd>
            </div>
          ))}
        </dl>
      ) : (
        <p className="px-3 py-2 text-sm text-n-9">o detalhe do que estiver selecionado vem aqui.</p>
      )}
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

  const repo = useOpenRepo().data;

  useDocumentTitle(repo?.name ?? null, repo?.branch ?? null);
  usePanelShortcuts();
  // Um lugar só liga o socket ao cache, na raiz da UI: dois pontos de escuta aplicariam o mesmo
  // evento duas vezes, e uma linha de log apareceria em dobro.
  useJobEvents();

  return (
    <div className="h-full flex flex-col bg-n-0 text-n-11">
      <TopBar repo={repo} />
      <div className="flex-1 min-h-0 flex">
        {!sidebarCollapsed && (
          <>
            <Sidebar width={sidebarWidth} repo={repo} />
            <Splitter panel="sidebar" edge="end" />
          </>
        )}
        <Center repo={repo} />
        {!detailCollapsed && (
          <>
            <Splitter panel="detail" edge="start" />
            <Detail width={detailWidth} repo={repo} />
          </>
        )}
      </div>
      <AskpassPrompt />
      <JobsPanel />
      <StatusBar />
    </div>
  );
}
