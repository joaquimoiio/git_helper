/* Navegador de pastas — o seletor de diretório que o navegador não tem.
 *
 * Uma coluna só, dirigida por teclado: ↓↑ move, → ou Enter entra, ← ou Backspace sobe, `.`
 * mostra ocultos. O mouse funciona, mas é o caminho secundário — quem abre repositório o dia
 * inteiro não tira a mão do teclado.
 *
 * Subir de nível restaura a seleção na pasta de onde se veio (`cameFrom`). Sem isso, descer
 * um nível para conferir e voltar joga o cursor no topo, e o usuário perde o lugar na lista.
 */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { keepPreviousData, useQuery } from "@tanstack/react-query";

import { api } from "../../lib/api";
import type { FsEntry, FsListing } from "../../lib/api-types";

function listPath(path: string | null, hidden: boolean): Promise<FsListing> {
  const params = new URLSearchParams();
  if (path) params.set("path", path);
  if (hidden) params.set("hidden", "true");

  const query = params.toString();
  return api.get<FsListing>(`/fs/list${query ? `?${query}` : ""}`);
}

/** Caminho relativo à raiz, que é o que o usuário reconhece. A raiz vira `~`. */
function breadcrumb(path: string, root: string): string {
  if (path === root) return "~";
  return `~${path.slice(root.length)}`;
}

export interface FolderBrowserProps {
  /** Chamado quando o usuário confirma uma pasta que já é repositório. */
  onPick?: (entry: FsEntry) => void;
  /** A pasta em que o navegador está, sempre que ela muda. É onde um `git init` aconteceria. */
  onPathChange?: (path: string) => void;
  /** Tecla `n`: criar um repositório aqui. */
  onNew?: () => void;
  /** Tecla `c`: clonar para dentro daqui. */
  onClone?: () => void;
}

export function FolderBrowser({ onPick, onPathChange, onNew, onClone }: FolderBrowserProps) {
  const [path, setPath] = useState<string | null>(null);
  const [hidden, setHidden] = useState(false);
  const [selected, setSelected] = useState(0);

  // Nome da pasta de onde subimos, por caminho de destino. `useRef` e não estado: mudar isto
  // nunca precisa redesenhar nada por si só.
  const cameFrom = useRef(new Map<string, string>());
  const listRef = useRef<HTMLUListElement>(null);

  const listing = useQuery({
    queryKey: ["fs", "list", path, hidden],
    queryFn: () => listPath(path, hidden),
    // Sem isto a lista pisca em branco a cada nível; com isto o painel só troca de conteúdo.
    placeholderData: keepPreviousData,
  });

  // Memoizado por causa do `?? []`: sem isto o fallback é um array novo a cada render, e o
  // efeito que reposiciona o cursor rodaria em todos eles.
  const entries = useMemo(() => listing.data?.entries ?? [], [listing.data]);

  // Ao chegar num diretório, o cursor vai para a pasta de onde viemos — ou para o topo.
  useEffect(() => {
    if (!listing.data) return;

    const name = cameFrom.current.get(listing.data.path);
    const index = name ? entries.findIndex((entry) => entry.name === name) : -1;
    setSelected(index >= 0 ? index : 0);
    // `entries` sai de `listing.data`, e o structural sharing do TanStack Query preserva a
    // referência quando o conteúdo não mudou — então um refetch que devolve a mesma lista não
    // faz o cursor pular do lugar onde o usuário o deixou.
  }, [listing.data, entries]);

  useEffect(() => {
    listRef.current?.children[selected]?.scrollIntoView({ block: "nearest" });
  }, [selected, entries.length]);

  useEffect(() => {
    if (listing.data) onPathChange?.(listing.data.path);
  }, [listing.data, onPathChange]);

  const enter = useCallback(
    (entry: FsEntry | undefined) => {
      if (!entry) return;
      setPath(entry.path);
    },
    [setPath],
  );

  const up = useCallback(() => {
    const data = listing.data;
    if (!data?.parent) return;

    // Guardado antes de mudar de caminho: é a pasta que o cursor tem que reencontrar lá em
    // cima.
    cameFrom.current.set(data.parent, data.path.slice(data.parent.length + 1));
    setPath(data.parent);
  }, [listing.data]);

  function onKeyDown(event: React.KeyboardEvent) {
    if (event.metaKey || event.ctrlKey || event.altKey) return;

    const last = entries.length - 1;
    const move = (index: number) => {
      event.preventDefault();
      setSelected(Math.min(last, Math.max(0, index)));
    };

    switch (event.key) {
      case "ArrowDown":
        return move(selected + 1);
      case "ArrowUp":
        return move(selected - 1);
      case "PageDown":
        return move(selected + 10);
      case "PageUp":
        return move(selected - 10);
      case "Home":
        return move(0);
      case "End":
        return move(last);
      case "ArrowRight":
        event.preventDefault();
        return enter(entries[selected]);
      case "ArrowLeft":
      case "Backspace":
        event.preventDefault();
        return up();
      case "Enter": {
        event.preventDefault();
        const entry = entries[selected];
        // Enter numa pasta que já é repositório confirma; nas outras, entra. É a diferença
        // entre "escolher" e "navegar", e evita um segundo atalho para a ação principal.
        if (entry?.isRepo && onPick) return onPick(entry);
        return enter(entry);
      }
      case ".":
        event.preventDefault();
        return setHidden((current) => !current);
      case "n":
        if (!onNew) return;
        event.preventDefault();
        return onNew();
      case "c":
        if (!onClone) return;
        event.preventDefault();
        return onClone();
    }
  }

  return (
    <div className="h-full flex flex-col min-h-0">
      <div className="flex items-baseline gap-2 h-8 px-3 border-b border-n-5 shrink-0">
        <span className="text-sm font-mono text-n-10 truncate">
          {listing.data ? breadcrumb(listing.data.path, listing.data.root) : "…"}
        </span>
        {listing.isFetching && <span className="text-xs text-n-8">carregando…</span>}
      </div>

      {listing.isError ? (
        <p className="px-3 py-2 text-sm text-error">{(listing.error as Error).message}</p>
      ) : (
        <ul
          ref={listRef}
          tabIndex={0}
          autoFocus
          role="listbox"
          aria-label="pastas"
          aria-activedescendant={entries[selected] ? `fs-${selected}` : undefined}
          onKeyDown={onKeyDown}
          className="flex-1 min-h-0 overflow-y-auto outline-none"
        >
          {entries.map((entry, index) => (
            <li
              key={entry.path}
              id={`fs-${index}`}
              role="option"
              aria-selected={index === selected}
              onClick={() => setSelected(index)}
              onDoubleClick={() => (entry.isRepo && onPick ? onPick(entry) : enter(entry))}
              className={`flex items-baseline gap-2 px-3 py-0.5 cursor-default transition-colors duration-(--duration-fast) ${
                index === selected ? "bg-n-4" : "hover:bg-n-3"
              }`}
            >
              <span className="text-sm text-n-8 shrink-0 w-3">{entry.isRepo ? "◆" : "▸"}</span>
              <span
                className={`text-base truncate ${entry.isRepo ? "text-n-11" : "text-n-10"} ${
                  entry.hidden ? "italic" : ""
                }`}
              >
                {entry.name}
              </span>
              {entry.isRepo && (
                <span className="ml-auto text-xs font-mono text-n-8 shrink-0">repo</span>
              )}
            </li>
          ))}

          {listing.data && entries.length === 0 && (
            <li className="px-3 py-1 text-sm text-n-8">nenhuma subpasta aqui</li>
          )}
        </ul>
      )}

      <div className="flex items-center gap-3 h-6 px-3 border-t border-n-5 text-xs font-mono text-n-8 shrink-0">
        <span>↓↑ mover</span>
        <span>→ entrar</span>
        <span>← subir</span>
        <span>enter abrir</span>
        <span>. ocultos{hidden ? " ✓" : ""}</span>
        {onNew && <span>n criar</span>}
        {onClone && <span>c clonar</span>}
      </div>
    </div>
  );
}
