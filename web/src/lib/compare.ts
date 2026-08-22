/* A comparação entre dois pontos quaisquer do histórico.
 *
 * Os dois lados são **estado de UI** (o que o usuário escolheu comparar) e ficam no Zustand,
 * como a seleção de repositório e a de commit. Não são persistidos: um `main`/`v1.0` guardado
 * entre boots apontaria para outra coisa depois de qualquer trabalho.
 *
 * `staleTime: Infinity` nas duas consultas — comparar dois pontos fixos do histórico dá sempre
 * o mesmo resultado. A exceção é comparar contra um **nome de branch** que andou; aí quem
 * invalida é o commit (`useCommit`), que já invalida `refs` e `log` pelo mesmo motivo.
 */

import { useQuery } from "@tanstack/react-query";
import { create } from "zustand";

import { api } from "./api";
import type { FileDiff, RangeDiff } from "./api-types";

interface CompareSelection {
  from: string;
  to: string;
  set: (side: "from" | "to", rev: string) => void;
  swap: () => void;
}

export const useCompareSelection = create<CompareSelection>()((set) => ({
  // O padrão compara o `HEAD` com o pai dele: é a comparação que sempre existe em qualquer
  // repositório com commit, e serve de exemplo do formato aceito.
  from: "HEAD~1",
  to: "HEAD",
  set: (side, rev) => set({ [side]: rev }),
  // Trocar os lados é o gesto mais comum depois de escolher: ver o que "voltaria" em vez do
  // que "iria".
  swap: () => set((state) => ({ from: state.to, to: state.from })),
}));

/** O diffstat acumulado. `enabled` só quando os dois lados têm algo escrito. */
export function useCompare(repoId: string, from: string, to: string) {
  return useQuery({
    queryKey: ["compare", repoId, from, to],
    queryFn: () =>
      api.get<RangeDiff>(
        `/repos/${repoId}/compare?from=${encodeURIComponent(from)}&to=${encodeURIComponent(to)}`,
      ),
    enabled: from.trim() !== "" && to.trim() !== "",
    staleTime: Infinity,
  });
}

/** Os hunks de um arquivo dentro da comparação, sob demanda. */
export function useCompareFileDiff(
  repoId: string,
  from: string,
  to: string,
  path: string | null,
) {
  return useQuery({
    queryKey: ["compare-diff", repoId, from, to, path],
    queryFn: () =>
      api.get<FileDiff>(
        `/repos/${repoId}/compare/diff?from=${encodeURIComponent(from)}&to=${encodeURIComponent(
          to,
        )}&path=${encodeURIComponent(path as string)}`,
      ),
    enabled: path !== null,
    staleTime: Infinity,
  });
}
