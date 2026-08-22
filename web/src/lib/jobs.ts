/* Jobs no cliente.
 *
 * O TanStack Query é o dono do estado; o WebSocket só o atualiza. É por isso que reconectar,
 * recarregar a aba ou perder eventos dá no mesmo: em todos os casos a saída é reconsultar
 * `GET /api/v1/jobs`, que devolve o estado completo.
 */

import { useEffect } from "react";
import { useMutation, useQuery, useQueryClient, type QueryClient } from "@tanstack/react-query";

import { api } from "./api";
import type { Accepted, CloneRequest, JobSnapshot, Repo, ServerMessage } from "./api-types";
import { repoKey, useRepoSelection } from "./repo";
import { channel } from "./ws";

const JOBS_KEY = ["jobs"] as const;

const jobKey = (jobId: string) => ["jobs", jobId] as const;

export function useJobs() {
  return useQuery({
    queryKey: JOBS_KEY,
    queryFn: () => api.get<JobSnapshot[]>("/jobs"),
    // Quem mantém isto fresco é o WebSocket. Sem ele o dado envelhece e a reconexão recarrega.
    staleTime: Infinity,
  });
}

export function useStartTestJob() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: () => api.post<Accepted>("/jobs/test"),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: JOBS_KEY }),
  });
}

export function useCloneRepo() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: CloneRequest) => api.post<Accepted>("/jobs/clone", request),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: JOBS_KEY }),
  });
}

export function useCancelJob() {
  return useMutation({
    // Nada de invalidar aqui: o `202` diz só que o pedido foi aceito. Quem conta o fim é o
    // evento `job.done`, depois de o processo morrer e a limpeza rodar.
    mutationFn: (jobId: string) => api.del(`/jobs/${jobId}`),
  });
}

/** Aplica um evento do socket ao cache, sem ida ao servidor. */
function apply(queryClient: QueryClient, message: ServerMessage) {
  const patch = (jobId: string, change: (job: JobSnapshot) => JobSnapshot) => {
    queryClient.setQueryData<JobSnapshot[]>(JOBS_KEY, (jobs) =>
      jobs?.map((job) => (job.jobId === jobId ? change(job) : job)),
    );
    queryClient.setQueryData<JobSnapshot>(jobKey(jobId), (job) => (job ? change(job) : job));
  };

  switch (message.type) {
    case "job.progress":
      return patch(message.jobId, (job) => ({ ...job, progress: message.progress }));

    case "job.log":
      return patch(message.jobId, (job) => ({ ...job, log: [...job.log, message.line] }));

    case "job.hit":
      return patch(message.jobId, (job) => ({ ...job, hits: [...job.hits, message.hit] }));

    case "job.askpass":
      return patch(message.jobId, (job) => ({
        ...job,
        pendingPrompt: { promptId: message.promptId, prompt: message.prompt },
      }));

    case "job.askpassClosed":
      return patch(message.jobId, (job) =>
        // Só fecha o pedido que está aberto: um evento atrasado não pode apagar o pedido
        // seguinte, que é o caso de quem errou a passphrase e o ssh perguntou de novo.
        job.pendingPrompt?.promptId === message.promptId
          ? { ...job, pendingPrompt: null }
          : job,
      );

    case "job.done": {
      const { job } = message;

      // Clone que deu certo já vem com o repositório aberto no `result` — abrir é o que o
      // usuário queria de verdade. Semear o cache aqui poupa um GET do que acabou de chegar.
      if (job.kind === "clone" && job.state === "done" && job.result) {
        const repo = job.result as Repo;
        queryClient.setQueryData(repoKey(repo.repoId), repo);
        queryClient.invalidateQueries({ queryKey: ["recents"] });
        queryClient.invalidateQueries({ queryKey: ["scan"] });
        useRepoSelection.getState().setRepoId(repo.repoId);
      }

      queryClient.setQueryData(jobKey(job.jobId), job);
      queryClient.setQueryData<JobSnapshot[]>(JOBS_KEY, (jobs) =>
        // Um job que terminou pode nunca ter passado por aqui (aba aberta depois de ele
        // começar), daí o `concat` em vez de só substituir.
        jobs?.some((existing) => existing.jobId === job.jobId)
          ? jobs.map((existing) => (existing.jobId === job.jobId ? job : existing))
          : [job, ...(jobs ?? [])],
      );
      return;
    }

    case "job.error":
      return patch(message.jobId, (job) => ({
        ...job,
        state: "error",
        message: message.message,
      }));

    case "resync":
      // Perdemos eventos. Não dá para remendar: joga fora e pergunta de novo.
      queryClient.invalidateQueries({ queryKey: JOBS_KEY });
      return;

    default:
      return;
  }
}

/**
 * Liga o socket ao cache. Montado uma vez, na raiz.
 *
 * Toda conexão nova invalida a lista de jobs: entre a queda e a volta pode ter acontecido
 * qualquer coisa, e o servidor é quem sabe o quê.
 */
export function useJobEvents() {
  const queryClient = useQueryClient();

  useEffect(() => {
    channel.subscribe("job");
    channel.connect();

    const stopListening = channel.listen((message) => apply(queryClient, message));
    const stopWatching = channel.whenOpen(() =>
      queryClient.invalidateQueries({ queryKey: JOBS_KEY }),
    );

    return () => {
      stopListening();
      stopWatching();
    };
  }, [queryClient]);
}
