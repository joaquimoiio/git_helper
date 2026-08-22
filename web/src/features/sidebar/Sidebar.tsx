/* A sidebar de refs: branches locais, remotas agrupadas por remote, tags e a pilha de stash.
 *
 * Uma lista só, achatada, com cabeçalho de grupo no meio — a mesma forma do `StatusPanel`, e
 * pelo mesmo motivo: o cursor atravessa os grupos com uma tecla só, e a busca filtra a lista
 * inteira sem que ninguém precise escolher em qual seção procurar.
 *
 * O agrupamento das remotas usa a **lista de remotes do git**, não a primeira barra do nome:
 * `origin/main` parte certo quase sempre, mas o git aceita remote com barra no nome, e quem
 * sabe onde o nome do remote termina é o próprio git.
 *
 * Enter leva ao commit da ponta no log. Trocar de branch é o Passo 59 — aqui a sidebar só
 * mostra e navega.
 */

import { useEffect, useMemo, useRef, useState } from "react";

import type { RefMarker, Remote, Repo, StashEntry } from "../../lib/api-types";
import { useCommitSelection } from "../../lib/commit";
import { useRefs, useRemotes, useStashes } from "../../lib/repo";

interface Row {
  /** Identidade estável da linha — tipo e nome, porque `main` existe em vários grupos. */
  id: string;
  /** O que aparece: nas remotas, sem o prefixo do remote (que já está no cabeçalho). */
  label: string;
  /** Contra o que a busca casa: sempre o nome inteiro, `origin/main` e não `main`. */
  search: string;
  oid: string;
  isHead: boolean;
  /** Coluna da direita: `stash@{0}` nos stashes, o oid curto no resto. */
  badge: string;
}

interface Group {
  key: string;
  title: string;
  /** URL de fetch, só nos grupos de remote — a mesma linha que o `git remote -v` mostra. */
  url: string | null;
  rows: Row[];
}

/** Ordem natural: `v2` antes de `v10`, que é o que quem olha uma lista de tags espera. */
function byLabel(a: Row, b: Row) {
  return a.label.localeCompare(b.label, undefined, { numeric: true });
}

/**
 * O remote a que uma remota pertence, pelo **nome mais longo** que a prefixa: com um remote
 * `origin` e outro `origin/backup` configurados, `origin/backup/main` é do segundo.
 */
function remoteOf(name: string, remotes: readonly Remote[]): Remote | null {
  let best: Remote | null = null;
  for (const remote of remotes) {
    if (!name.startsWith(`${remote.name}/`)) continue;
    if (best === null || remote.name.length > best.name.length) best = remote;
  }
  return best;
}

function build(
  refs: readonly RefMarker[],
  remotes: readonly Remote[],
  stashes: readonly StashEntry[],
): Group[] {
  const locals: Row[] = [];
  const tags: Row[] = [];
  // Remota cujo prefixo não bate com remote nenhum: acontece quando alguém remove o remote e
  // deixa as `refs/remotes/<nome>/…` para trás. Esconder seria mentir sobre o que existe.
  const orphans: Row[] = [];
  const byRemote = new Map<string, Row[]>(remotes.map((remote) => [remote.name, []]));

  for (const marker of refs) {
    const row: Row = {
      id: `${marker.kind}:${marker.name}`,
      label: marker.name,
      search: marker.name,
      oid: marker.commit,
      isHead: marker.isHead,
      badge: marker.commit.slice(0, 7),
    };

    switch (marker.kind) {
      case "branch":
      case "head":
        locals.push(row);
        break;
      case "tag":
        tags.push(row);
        break;
      case "remote": {
        const remote = remoteOf(marker.name, remotes);
        if (!remote) {
          orphans.push(row);
          break;
        }
        // O nome do remote já está no cabeçalho do grupo: repeti-lo em cada linha gastaria
        // metade da largura da sidebar com a mesma palavra.
        const label = marker.name.slice(remote.name.length + 1);
        byRemote.get(remote.name)?.push({ ...row, label });
        break;
      }
    }
  }

  const groups: Group[] = [
    { key: "branches", title: "branches", url: null, rows: locals.sort(byLabel) },
  ];

  // Todo remote configurado ganha cabeçalho, mesmo sem nenhuma remota buscada ainda: a sidebar
  // é também onde se vê que o remote existe (e o Passo 62 vai gerenciá-los daqui).
  for (const remote of remotes) {
    groups.push({
      key: `remote:${remote.name}`,
      title: remote.name,
      url: remote.fetchUrl,
      rows: (byRemote.get(remote.name) ?? []).sort(byLabel),
    });
  }

  if (orphans.length > 0) {
    const title = "remotas sem remote";
    groups.push({ key: "orfas", title, url: null, rows: orphans.sort(byLabel) });
  }

  if (tags.length > 0) {
    groups.push({ key: "tags", title: "tags", url: null, rows: tags.sort(byLabel) });
  }

  if (stashes.length > 0) {
    groups.push({
      key: "stashes",
      title: "stashes",
      url: null,
      rows: stashes.map((stash) => ({
        id: `stash:${stash.index}`,
        label: stash.message,
        search: stash.message,
        oid: stash.oid,
        isHead: false,
        badge: `stash@{${stash.index}}`,
      })),
    });
  }

  return groups;
}

/** Filtra por substring, sem fuzzy: o fuzzy é do Passo 59, e é sobre trocar de branch. */
function filter(groups: readonly Group[], query: string): Group[] {
  const needle = query.trim().toLowerCase();
  if (needle === "") return groups.slice();

  return groups
    .map((group) => ({
      ...group,
      rows: group.rows.filter((row) => row.search.toLowerCase().includes(needle)),
    }))
    .filter((group) => group.rows.length > 0);
}

function RefTree({ repoId, onReveal }: { repoId: string; onReveal: () => void }) {
  const refs = useRefs(repoId);
  const remotes = useRemotes(repoId);
  const stashes = useStashes(repoId);
  const select = useCommitSelection((state) => state.select);

  const [query, setQuery] = useState("");
  const [cursor, setCursor] = useState(0);
  const listRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);
  const cursorRef = useRef<HTMLDivElement>(null);

  const groups = useMemo(
    () => filter(build(refs.data ?? [], remotes.data ?? [], stashes.data ?? []), query),
    [refs.data, remotes.data, stashes.data, query],
  );
  const rows = useMemo(() => groups.flatMap((group) => group.rows), [groups]);

  // Ajuste durante o render, não em efeito: a lista encolhe a cada tecla da busca, e um cursor
  // fora do fim destacaria uma linha que não existe até o efeito rodar.
  const at = rows.length === 0 ? 0 : Math.min(cursor, rows.length - 1);
  if (at !== cursor) setCursor(at);

  useEffect(() => {
    cursorRef.current?.scrollIntoView({ block: "nearest" });
  }, [at]);

  function reveal(index: number) {
    const row = rows[index];
    if (!row) return;

    setCursor(index);
    // Stash é um commit fora do histórico: o detalhe abre normalmente, mas nenhuma linha do log
    // vai acender. Aplicar e soltar stash são do Passo 60.
    select(row.oid);
    onReveal();
  }

  function move(index: number) {
    setCursor(Math.min(Math.max(index, 0), Math.max(rows.length - 1, 0)));
  }

  function onKeyDown(event: React.KeyboardEvent) {
    if (event.metaKey || event.ctrlKey || event.altKey) return;

    switch (event.key) {
      case "ArrowDown":
      case "j":
        event.preventDefault();
        return move(at + 1);
      case "ArrowUp":
      case "k":
        event.preventDefault();
        return move(at - 1);
      case "Home":
        event.preventDefault();
        return move(0);
      case "End":
        event.preventDefault();
        return move(rows.length - 1);
      case "Enter":
        event.preventDefault();
        return reveal(at);
      case "/":
        // A busca é o gesto seguinte de quem está na lista e não achou o que queria.
        event.preventDefault();
        searchRef.current?.focus();
        return;
      case "Escape":
        event.preventDefault();
        setQuery("");
        return;
    }
  }

  function onSearchKeyDown(event: React.KeyboardEvent) {
    if (event.metaKey || event.ctrlKey || event.altKey) return;

    if (event.key === "ArrowDown") {
      event.preventDefault();
      listRef.current?.focus();
      return;
    }
    if (event.key === "Enter") {
      // Digitar três letras e dar Enter já leva à primeira ponta que sobrou — sem passar
      // pela lista no meio do caminho.
      event.preventDefault();
      listRef.current?.focus();
      reveal(at);
      return;
    }
    if (event.key === "Escape") {
      event.preventDefault();
      setQuery("");
    }
  }

  const failed = refs.error ?? remotes.error ?? stashes.error;

  return (
    <>
      <div className="px-3 py-1.5 border-b border-n-5">
        <input
          ref={searchRef}
          value={query}
          onChange={(event) => {
            setQuery(event.target.value);
            setCursor(0);
          }}
          onKeyDown={onSearchKeyDown}
          placeholder="filtrar refs"
          aria-label="filtrar refs"
          className="w-full px-1.5 py-0.5 text-sm font-mono bg-n-2 border border-n-5 rounded-sm text-n-11 placeholder:text-n-8 outline-none focus:border-n-7 transition-colors duration-(--duration-fast)"
        />
      </div>
      {failed && <p className="px-3 py-1 text-sm text-error bg-error-bg">{failed.message}</p>}
      <div
        ref={listRef}
        tabIndex={0}
        role="listbox"
        aria-label="refs do repositório"
        onKeyDown={onKeyDown}
        className="outline-none pb-2"
      >
        {refs.isPending && <p className="px-3 py-2 text-sm text-n-8">carregando…</p>}
        {!refs.isPending && rows.length === 0 && (
          <p className="px-3 py-2 text-sm text-n-8">
            {query.trim() === "" ? "nenhuma ref neste repositório." : "nada com esse nome."}
          </p>
        )}
        {groups.map((group) => (
          <section key={group.key}>
            <h2 className="flex items-baseline gap-2 px-3 pt-2 pb-0.5">
              <span className="text-xs uppercase tracking-[0.1em] text-n-8 shrink-0">
                {group.title}
              </span>
              <span className="text-xs font-mono text-n-8 shrink-0">{group.rows.length}</span>
              {group.url && (
                <span className="text-xs font-mono text-n-7 truncate min-w-0" title={group.url}>
                  {group.url}
                </span>
              )}
            </h2>
            {group.rows.map((row) => {
              const index = rows.indexOf(row);
              const isCursor = index === at;

              return (
                <div
                  key={row.id}
                  ref={isCursor ? cursorRef : undefined}
                  role="option"
                  aria-selected={isCursor}
                  onClick={() => reveal(index)}
                  className={`flex items-baseline gap-2 px-3 py-0.5 cursor-default transition-colors duration-(--duration-fast) ${
                    isCursor ? "bg-n-4" : "hover:bg-n-3"
                  }`}
                >
                  {/* Sem arco-íris: a ponta do `HEAD` se distingue por peso e por um marcador
                      de largura fixa, para os nomes continuarem alinhados. */}
                  <span className="w-2 shrink-0 text-sm font-mono text-n-9 select-none">
                    {row.isHead ? "●" : ""}
                  </span>
                  <span
                    className={`flex-1 min-w-0 truncate text-sm font-mono ${
                      row.isHead ? "text-n-11 font-medium" : "text-n-10"
                    }`}
                    title={row.search}
                  >
                    {row.label}
                  </span>
                  <span className="shrink-0 text-xs font-mono text-n-8">{row.badge}</span>
                </div>
              );
            })}
          </section>
        ))}
      </div>
    </>
  );
}

export function Sidebar({
  width,
  repo,
  onReveal,
}: {
  width: number;
  repo: Repo | undefined;
  onReveal: () => void;
}) {
  return (
    <aside className="shrink-0 overflow-y-auto border-r border-n-5 bg-n-1" style={{ width }}>
      {repo ? (
        <>
          <section className="px-3 py-2 border-b border-n-5">
            <p className="text-base text-n-11 truncate">{repo.name}</p>
            <p className="mt-0.5 text-xs font-mono text-n-8 break-all">{repo.path}</p>
          </section>
          {/* `key` no repoId: trocar de repositório tem que zerar busca e cursor, senão o
              filtro do repositório anterior esconderia as refs do novo. */}
          <RefTree key={repo.repoId} repoId={repo.repoId} onReveal={onReveal} />
        </>
      ) : (
        <p className="px-3 py-2 text-sm text-n-9">
          escolha uma pasta no centro. as marcadas com ◆ já são repositórios.
        </p>
      )}
    </aside>
  );
}
