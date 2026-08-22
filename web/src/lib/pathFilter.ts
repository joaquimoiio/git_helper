/* Filtro por caminho (Passo 45): dispara o job `path-filter` e lê o resultado do mesmo cache
 * de jobs que o `JobsPanel` já usa — o indicador de progresso e o botão de cancelar do
 * `JobsPanel` já funcionam para este job de graça, sem nada escrito aqui para isso.
 */

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";

import { api } from "./api";
import type { Accepted, JobSnapshot, SearchHit } from "./api-types";
import { useJobs } from "./jobs";

export function useStartPathFilter(repoId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (path: string) => api.post<Accepted>("/jobs/path-filter", { repoId, path }),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: ["jobs"] }),
  });
}

/** O snapshot vivo de um job de filtro, seguido pelo mesmo canal que alimenta o `JobsPanel`. */
export function usePathFilterJob(jobId: string | null): JobSnapshot | undefined {
  const jobs = useJobs();
  return jobId ? jobs.data?.find((job) => job.jobId === jobId) : undefined;
}

/** `job.result` chega como `unknown` — só este job sabe que é `{ commits: SearchHit[] }`. */
export function pathFilterHits(job: JobSnapshot | undefined): SearchHit[] {
  const result = job?.result as { commits?: SearchHit[] } | undefined;
  return result?.commits ?? [];
}

/**
 * Autocomplete: nomes que começam com `prefix` no nível de pasta que ele indica. `staleTime`
 * curto, não `Infinity` como o resto do log — a árvore muda a cada commit novo, diferente de
 * um commit já escrito, que nunca muda.
 */
export function usePathAutocomplete(repoId: string, prefix: string) {
  return useQuery({
    queryKey: ["paths", repoId, prefix],
    queryFn: () => api.get<string[]>(`/repos/${repoId}/paths?prefix=${encodeURIComponent(prefix)}`),
    staleTime: 10_000,
  });
}
