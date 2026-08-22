/* Busca por mensagem e autor (FTS5, Passo 43).
 *
 * O debounce mora aqui, não no componente: é o que faz `useSearch` já receber uma query
 * estável, sem cada tecla disparar um request que a próxima tecla imediatamente invalidaria.
 */

import { useEffect, useState } from "react";
import { useQuery } from "@tanstack/react-query";

import { api } from "./api";
import type { SearchHit } from "./api-types";

/* Curto o bastante para parecer instantâneo, longo o bastante para não disparar um request
 * por tecla numa digitação normal — "incremental a cada tecla" é sobre o resultado parecer
 * ao vivo, não sobre literalmente uma requisição por evento de teclado. */
export const SEARCH_DEBOUNCE_MS = 150;

export function useDebouncedValue<T>(value: T, delayMs: number): T {
  const [debounced, setDebounced] = useState(value);

  useEffect(() => {
    const timer = setTimeout(() => setDebounced(value), delayMs);
    return () => clearTimeout(timer);
  }, [value, delayMs]);

  return debounced;
}

/**
 * Resultados da busca. `query` vazia (ou só espaço) não consulta o servidor — é o estado de
 * "sem filtro", tratado aqui e não só na rota, para nem abrir uma requisição à toa.
 */
export function useSearch(repoId: string, query: string) {
  const trimmed = query.trim();

  return useQuery({
    queryKey: ["search", repoId, trimmed],
    queryFn: () => api.get<SearchHit[]>(`/repos/${repoId}/search?q=${encodeURIComponent(trimmed)}`),
    enabled: trimmed.length > 0,
    // Um resultado de busca não fica velho sozinho — o índice é que muda (Passo 42), e nada
    // aqui reage a isso ainda. Refazer a busca a cada foco de janela seria trabalho à toa.
    staleTime: Infinity,
  });
}
