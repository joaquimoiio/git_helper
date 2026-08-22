/* O commit selecionado, e o detalhe completo dele.
 *
 * Seleção de commit é um dimensão de UI à parte da seleção de repositório (`useRepoSelection`):
 * clicar num pai no painel de detalhe troca o *assunto* do painel sem mexer na linha destacada
 * na lista — "parents clicáveis" é navegar o histórico ali mesmo, não pular o cursor do log.
 * Guardar os dois na mesma store obrigaria um a saber do outro.
 */

import { useQuery } from "@tanstack/react-query";
import { create } from "zustand";

import { api } from "./api";
import type { CommitDetail, FileDiff } from "./api-types";

interface CommitSelection {
  oid: string | null;
  select: (oid: string | null) => void;
}

export const useCommitSelection = create<CommitSelection>()((set) => ({
  oid: null,
  select: (oid) => set({ oid }),
}));

/** Detalhe completo do commit selecionado. `oid` ausente é "nada selecionado ainda". */
export function useCommitDetail(repoId: string, oid: string | null) {
  return useQuery({
    queryKey: ["commit", repoId, oid],
    queryFn: () => api.get<CommitDetail>(`/repos/${repoId}/commits/${oid}`),
    enabled: oid !== null,
    // Um commit não muda depois de escrito — diferente do log (que ganha commits novos) e das
    // refs (que apontam para outro lugar), isto nunca precisa invalidar.
    staleTime: Infinity,
  });
}

/**
 * O diff de **um** arquivo dentro do commit selecionado. Sob demanda: um commit pode tocar
 * centenas de arquivos, e a lista de arquivos do Passo 40 já mostra o diffstat sem precisar
 * do patch inteiro — só busca isto quando o usuário abre um arquivo específico.
 */
export function useFileDiff(repoId: string, oid: string, path: string | null) {
  return useQuery({
    queryKey: ["diff", repoId, oid, path],
    queryFn: () =>
      api.get<FileDiff>(`/repos/${repoId}/commits/${oid}/diff?path=${encodeURIComponent(path as string)}`),
    enabled: path !== null,
    staleTime: Infinity,
  });
}

/** `time` em segundos desde a época, `offsetMinutes` do fuso de quem assinou. */
export function formatSignatureDate(time: number, offsetMinutes: number): string {
  // Desloca para o fuso de quem assinou e formata como se fosse UTC — é o único jeito de o
  // `Intl` mostrar um fuso arbitrário sem duplicar o deslocamento por cima do fuso local do
  // navegador.
  const shifted = new Date((time + offsetMinutes * 60) * 1000);
  const date = shifted.toLocaleString("pt-BR", {
    timeZone: "UTC",
    dateStyle: "medium",
    timeStyle: "short",
  });

  const sign = offsetMinutes >= 0 ? "+" : "-";
  const absMinutes = Math.abs(offsetMinutes);
  const hh = String(Math.floor(absMinutes / 60)).padStart(2, "0");
  const mm = String(absMinutes % 60).padStart(2, "0");

  return `${date} (UTC${sign}${hh}:${mm})`;
}
