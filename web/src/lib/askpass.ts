/* Pedidos de senha.
 *
 * Não há store próprio: o pedido em aberto é um campo do estado do job (`pendingPrompt`), que o
 * servidor devolve tanto pelo WebSocket quanto pelo `GET /api/v1/jobs`. É isso que faz
 * recarregar a aba no meio de um clone por SSH mostrar o campo de senha de novo, em vez de
 * deixar o job esperando uma resposta que ninguém sabe mais que foi pedida.
 *
 * O segredo nunca é guardado: existe no campo do formulário, é enviado, e some.
 */

import { useMutation } from "@tanstack/react-query";

import { api } from "./api";
import type { PendingPrompt } from "./api-types";
import { useJobs } from "./jobs";

export interface OpenPrompt extends PendingPrompt {
  jobId: string;
}

/** O primeiro pedido em aberto, se houver. Um de cada vez — senha é uma coisa modal. */
export function usePendingPrompt(): OpenPrompt | undefined {
  const jobs = useJobs();

  const job = (jobs.data ?? []).find(
    (job) => job.state === "running" && job.pendingPrompt !== null,
  );

  return job?.pendingPrompt ? { jobId: job.jobId, ...job.pendingPrompt } : undefined;
}

export function useAnswerAskpass() {
  return useMutation({
    mutationFn: ({ jobId, promptId, secret }: OpenPrompt & { secret: string }) =>
      api.post(`/jobs/${jobId}/askpass`, { promptId, secret }),
  });
}
