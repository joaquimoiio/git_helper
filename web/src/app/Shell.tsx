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
import { CompareView } from "../features/compare/CompareView";
import { CommitDetail } from "../features/log/CommitDetail";
import { Log } from "../features/log/Log";
import { Sidebar } from "../features/sidebar/Sidebar";
import { StatusDetail } from "../features/status/StatusDetail";
import { StatusPanel } from "../features/status/StatusPanel";
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

/* O centro tem três assuntos — o histórico, o trabalho local e a comparação entre dois pontos —
 * e o `CLAUDE.md` os põe no mesmo lugar ("centro (log ou status)"). Abas, não painéis: os três
 * querem a largura inteira e o teclado inteiro, e nenhum é consultado enquanto outro é usado. */
function CenterTabs({ view, onChange }: { view: CenterView; onChange: (view: CenterView) => void }) {
  const tab = (value: CenterView, label: string) => (
    <button
      type="button"
      onClick={() => onChange(value)}
      className={`text-xs px-1.5 py-0.5 rounded-sm border transition-colors duration-(--duration-fast) ${
        view === value ? "border-n-7 bg-n-4 text-n-11" : "border-n-6 text-n-9 hover:text-n-11"
      }`}
    >
      {label}
    </button>
  );

  return (
    <div className="flex items-center gap-1 shrink-0">
      {tab("log", "log")}
      {tab("status", "status")}
      {tab("compare", "comparar")}
    </div>
  );
}

function Center({
  repo,
  view,
  onView,
}: {
  repo: Repo | undefined;
  view: CenterView;
  onView: (view: CenterView) => void;
}) {
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
      <main className="flex-1 min-w-0 min-h-0 flex flex-col bg-n-0">
        <div className="flex items-center gap-3 px-3 py-2 border-b border-n-5 shrink-0">
          <div className="min-w-0">
            <p className="text-base text-n-11 truncate">{repo.name}</p>
            <p className="mt-0.5 text-sm font-mono text-n-9 truncate">
              {repo.head.kind === "unborn"
                ? `${repo.branch} · nenhum commit ainda`
                : `${repo.branch} · ${repo.head.commit.slice(0, 12)}`}
            </p>
          </div>
          <span className="flex-1" />
          <CenterTabs view={view} onChange={onView} />
        </div>
        {view === "log" && <Log repoId={repo.repoId} />}
        {view === "status" && <StatusPanel repoId={repo.repoId} />}
        {view === "compare" && <CompareView repoId={repo.repoId} />}
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

function Detail({
  width,
  repo,
  view,
}: {
  width: number;
  repo: Repo | undefined;
  view: CenterView;
}) {
  return (
    <aside className="shrink-0 overflow-y-auto border-l border-n-5 bg-n-1" style={{ width }}>
      {repo && view === "status" ? (
        // O detalhe é sempre o do assunto do centro: com o status à mostra, o que interessa é o
        // diff do arquivo sob o cursor, não o commit que ficou selecionado no log.
        <StatusDetail repoId={repo.repoId} />
      ) : repo ? (
        // Repositório sem commit nenhum (`unborn`): não há o que selecionar no log, então não
        // há detalhe de commit para mostrar — só o repositório em si.
        repo.head.kind === "unborn" ? (
          <p className="px-3 py-2 text-sm text-n-9">sem commits ainda neste repositório.</p>
        ) : (
          <CommitDetail repoId={repo.repoId} />
        )
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

/** O que o painel central mostra. Não é persistido: quem abre o app quer ver o histórico. */
type CenterView = "log" | "status" | "compare";

export function Shell() {
  const sidebarWidth = useLayout((state) => state.sidebarWidth);
  const detailWidth = useLayout((state) => state.detailWidth);
  const sidebarCollapsed = useLayout((state) => state.sidebarCollapsed);
  const detailCollapsed = useLayout((state) => state.detailCollapsed);

  const repo = useOpenRepo().data;
  const [view, setView] = useState<CenterView>("log");

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
            {/* Escolher uma ref na sidebar leva ao commit dela — e isso só faz sentido com o
                log à mostra, então o centro volta para ele. */}
            <Sidebar width={sidebarWidth} repo={repo} onReveal={() => setView("log")} />
            <Splitter panel="sidebar" edge="end" />
          </>
        )}
        <Center repo={repo} view={view} onView={setView} />
        {!detailCollapsed && (
          <>
            <Splitter panel="detail" edge="start" />
            <Detail width={detailWidth} repo={repo} view={view} />
          </>
        )}
      </div>
      <AskpassPrompt />
      <JobsPanel />
      <StatusBar />
    </div>
  );
}
