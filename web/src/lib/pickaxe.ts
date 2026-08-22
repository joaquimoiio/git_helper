/* Busca por conteúdo — pickaxe (Passo 46): dispara o job `pickaxe` e lê tanto a cauda ao vivo
 * (`hits`, evento por evento) quanto o resultado final, do mesmo cache de jobs que o
 * `JobsPanel` já usa — igual ao filtro por caminho (Passo 45), só que aqui a lista cresce
 * *enquanto* o job roda, não só quando termina.
 */

import { useMutation, useQueryClient } from "@tanstack/react-query";

import { api } from "./api";
import type { Accepted, JobSnapshot, SearchHit } from "./api-types";
import { useJobs } from "./jobs";

export type PickaxeMode = "string" | "regex";

export function useStartPickaxe(repoId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: ({ mode, value }: { mode: PickaxeMode; value: string }) =>
      api.post<Accepted>("/jobs/pickaxe", { repoId, mode, value }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["jobs"] }),
  });
}

/** O snapshot vivo de um job de pickaxe, seguido pelo mesmo canal que alimenta o `JobsPanel`. */
export function usePickaxeJob(jobId: string | null): JobSnapshot | undefined {
  const jobs = useJobs();
  return jobId ? jobs.data?.find((job) => job.jobId === jobId) : undefined;
}

/**
 * Enquanto o job roda: a cauda ao vivo (`hits`, capada nos mais recentes — é o que faz o
 * grafo "filtrar conforme os oids chegam"). Terminado: a lista completa do `result`, sem
 * corte nenhum.
 */
export function pickaxeHits(job: JobSnapshot | undefined): SearchHit[] {
  if (job?.state === "done") {
    const result = job.result as { commits?: SearchHit[] } | undefined;
    return result?.commits ?? [];
  }

  return (job?.hits as SearchHit[] | undefined) ?? [];
}
