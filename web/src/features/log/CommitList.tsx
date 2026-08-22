/* A lista de commits: uma coluna densa, dirigida por teclado como o `FolderBrowser`, com o
 * grafo (`LogGraph`) sobreposto na faixa esquerda.
 *
 * `TanStack Virtual` é o que faz 100k commits caberem no orçamento de scroll — só as linhas
 * dentro (e um pouco além) da viewport existem no DOM. A lista de texto continua DOM
 * (selecionável, acessível); só o grafo é pintado num `<canvas>`.
 */

import { useCallback, useEffect, useMemo, useRef, useState } from "react";
import { useVirtualizer } from "@tanstack/react-virtual";

import type { RefMarker } from "../../lib/api-types";
import { useCommitSelection } from "../../lib/commit";
import { relativeTime, useLog } from "../../lib/log";
import { useRefs } from "../../lib/repo";
import { LogGraph } from "./LogGraph";
import { RefBadges } from "./RefBadges";

/* `px-3 py-0.5 text-sm` do resto do app: 18px de linha + 2×2px de padding. Linha fixa, não
 * medida — é o que permite ao virtualizador pular direto para qualquer índice sem layout. */
const ROW_HEIGHT = 22;

/* Largura de uma coluna do grafo. Ponto fino, aresta fina — a faixa só precisa caber os
 * traços, não decoração. */
const LANE_WIDTH = 14;

/* Buscar a próxima página um pouco antes do fim, não exatamente nele: com 500 commits por
 * página e rolagem rápida, esperar o último pixel deixaria um vão em branco visível. */
const PREFETCH_MARGIN = 100;

export function CommitList({ repoId }: { repoId: string }) {
  const { commits, isPending, isError, error, hasNextPage, isFetchingNextPage, fetchNextPage } =
    useLog(repoId);
  const refs = useRefs(repoId);
  const [selected, setSelected] = useState(0);
  const parentRef = useRef<HTMLDivElement>(null);
  const selectCommit = useCommitSelection((state) => state.select);

  // oid → pontas que apontam para ele. Recalculado só quando a lista de refs muda (criar uma
  // branch, fazer um checkout por fora) — não é grande (dezenas, no máximo centenas) e não
  // cresce com o log.
  const refsByCommit = useMemo(() => {
    const map = new Map<string, RefMarker[]>();
    for (const marker of refs.data ?? []) {
      const existing = map.get(marker.commit);
      if (existing) existing.push(marker);
      else map.set(marker.commit, [marker]);
    }
    return map;
  }, [refs.data]);

  const virtualizer = useVirtualizer({
    count: commits.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 16,
  });

  const items = virtualizer.getVirtualItems();

  // Largura da faixa do grafo: a maior coluna entre as já vistas, seja como linha de um
  // commit ou como destino de uma aresta ainda não emitida. Só recalcula quando uma página
  // nova chega, não a cada quadro de rolagem.
  const gutterWidth = useMemo(() => {
    let maxLane = 0;
    for (const commit of commits) {
      if (commit.lane > maxLane) maxLane = commit.lane;
      for (const parentLane of commit.parentLanes) {
        if (parentLane !== null && parentLane > maxLane) maxLane = parentLane;
      }
    }
    return (maxLane + 1) * LANE_WIDTH + LANE_WIDTH / 2;
  }, [commits]);

  // A fronteira de páginas é opaca ao virtualizador: ele só sabe que existem N linhas. Quando
  // a rolagem chega perto do fim do que já carregamos, pede a próxima página.
  useEffect(() => {
    const last = items.at(-1);
    if (!last) return;
    if (last.index < commits.length - 1) return;
    if (!hasNextPage || isFetchingNextPage) return;

    fetchNextPage();
  }, [items, commits.length, hasNextPage, isFetchingNextPage, fetchNextPage]);

  useEffect(() => {
    // Trocar de repositório reseta a lista: o cursor do log anterior não significa nada aqui.
    setSelected(0);
    virtualizer.scrollToIndex(0);
  }, [repoId, virtualizer]);

  // O painel de detalhe segue o cursor do log. Só nesta direção: clicar num pai lá no painel
  // troca o assunto dele sem mover este cursor, e por isso não entra nas dependências daqui —
  // senão o próximo movimento de teclado desfaria a navegação por pai.
  useEffect(() => {
    selectCommit(commits[selected]?.oid ?? null);
  }, [selected, commits, selectCommit]);

  const move = useCallback(
    (index: number) => {
      const last = commits.length - 1;
      const clamped = Math.min(Math.max(index, 0), Math.max(last, 0));
      setSelected(clamped);
      virtualizer.scrollToIndex(clamped, { align: "auto" });

      // Perto do fim do carregado, mas ainda tem página: garante que mover o cursor não
      // rode na frente do que a rolagem buscaria sozinha.
      if (clamped >= commits.length - PREFETCH_MARGIN / ROW_HEIGHT && hasNextPage && !isFetchingNextPage) {
        fetchNextPage();
      }
    },
    [commits.length, hasNextPage, isFetchingNextPage, fetchNextPage, virtualizer],
  );

  function onKeyDown(event: React.KeyboardEvent) {
    if (event.metaKey || event.ctrlKey || event.altKey) return;
    if (commits.length === 0) return;

    const viewport = Math.max(1, Math.floor((parentRef.current?.clientHeight ?? 0) / ROW_HEIGHT));

    switch (event.key) {
      case "ArrowDown":
      case "j":
        event.preventDefault();
        return move(selected + 1);
      case "ArrowUp":
      case "k":
        event.preventDefault();
        return move(selected - 1);
      case "PageDown":
        event.preventDefault();
        return move(selected + viewport);
      case "PageUp":
        event.preventDefault();
        return move(selected - viewport);
      case "Home":
        event.preventDefault();
        return move(0);
      case "End":
        event.preventDefault();
        return move(commits.length - 1);
    }
  }

  if (isPending) {
    return <p className="px-3 py-2 text-sm text-n-8">carregando…</p>;
  }

  if (isError) {
    return <p className="px-3 py-2 text-sm text-error">{(error as Error).message}</p>;
  }

  if (commits.length === 0) {
    return <p className="px-3 py-2 text-sm text-n-8">nenhum commit ainda</p>;
  }

  return (
    <div className="relative flex-1 min-h-0">
      <LogGraph
        commits={commits}
        items={items}
        scrollOffset={virtualizer.scrollOffset ?? 0}
        rowHeight={ROW_HEIGHT}
        laneWidth={LANE_WIDTH}
        gutterWidth={gutterWidth}
        selectedIndex={selected}
      />
      <div
        ref={parentRef}
        tabIndex={0}
        role="listbox"
        aria-label="commits"
        aria-activedescendant={`commit-${selected}`}
        onKeyDown={onKeyDown}
        className="h-full overflow-y-auto outline-none"
      >
        <div style={{ height: virtualizer.getTotalSize() }} className="relative">
          {items.map((item) => {
            const commit = commits[item.index];
            if (!commit) return null;

            return (
              <div
                key={commit.oid}
                id={`commit-${item.index}`}
                role="option"
                aria-selected={item.index === selected}
                onClick={() => setSelected(item.index)}
                className={`absolute inset-x-0 flex items-baseline gap-3 pr-3 cursor-default transition-colors duration-(--duration-fast) ${
                  item.index === selected ? "bg-n-4" : "hover:bg-n-3"
                }`}
                style={{ top: item.start, height: item.size, paddingLeft: gutterWidth + 8 }}
              >
                <span className="text-sm font-mono text-n-8 shrink-0">{commit.oid.slice(0, 7)}</span>
                <RefBadges markers={refsByCommit.get(commit.oid)} />
                <span className="text-sm text-n-11 truncate flex-1 min-w-0">{commit.summary}</span>
                <span className="text-sm text-n-9 truncate shrink-0 max-w-32">{commit.author}</span>
                <span className="text-xs text-n-8 shrink-0 w-20 text-right">
                  {relativeTime(commit.time)}
                </span>
              </div>
            );
          })}
        </div>
      </div>
    </div>
  );
}
