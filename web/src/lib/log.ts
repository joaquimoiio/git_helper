/* O log como o cliente o vê: páginas de 500 commits encadeadas pelo cursor opaco do
 * servidor, achatadas numa lista só para a lista virtualizada consumir.
 *
 * `getNextPageParam` é o que faz o `useInfiniteQuery` saber pedir a próxima página — devolver
 * `undefined` é o sinal de "acabou" que a lib espera, daí o `?? undefined` em cima do `null`
 * que o servidor manda.
 */

import { useInfiniteQuery } from "@tanstack/react-query";
import { useMemo } from "react";

import { api } from "./api";
import type { Commit, LogPage } from "./api-types";

function fetchLog(repoId: string, cursor: string | undefined): Promise<LogPage> {
  const params = new URLSearchParams();
  if (cursor) params.set("cursor", cursor);

  const query = params.toString();
  return api.get<LogPage>(`/repos/${repoId}/log${query ? `?${query}` : ""}`);
}

export function useLog(repoId: string | undefined) {
  const query = useInfiniteQuery({
    queryKey: ["log", repoId],
    queryFn: ({ pageParam }) => fetchLog(repoId as string, pageParam),
    initialPageParam: undefined as string | undefined,
    getNextPageParam: (page: LogPage) => page.nextCursor ?? undefined,
    enabled: repoId !== undefined,
    // O log não muda por baixo dos pés enquanto o usuário rola (mudanças de repo chegam por
    // WS no Bloco E); não vale reconsultar do zero a cada foco de janela.
    staleTime: Infinity,
  });

  // Memoizado: sem isto, cada render do consumidor recriaria o array e a lista virtualizada
  // perderia a referência estável de que precisa para não redesenhar tudo.
  const commits = useMemo<Commit[]>(
    () => query.data?.pages.flatMap((page) => page.commits) ?? [],
    [query.data],
  );

  return { ...query, commits };
}

const RELATIVE = new Intl.RelativeTimeFormat("pt-BR", { numeric: "auto" });
const UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ["year", 31_536_000],
  ["month", 2_592_000],
  ["day", 86_400],
  ["hour", 3_600],
  ["minute", 60],
];

/** `time` em segundos desde a época (é como o `Commit` chega do servidor). */
export function relativeTime(time: number, now = Date.now()): string {
  const deltaSeconds = time - Math.floor(now / 1000);

  for (const [unit, secondsInUnit] of UNITS) {
    if (Math.abs(deltaSeconds) >= secondsInUnit) {
      return RELATIVE.format(Math.round(deltaSeconds / secondsInUnit), unit);
    }
  }
  return RELATIVE.format(Math.round(deltaSeconds / 1), "second");
}
