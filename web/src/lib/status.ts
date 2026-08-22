/* O status do working tree, e as duas mutações que mexem nele.
 *
 * Diferente do log e das refs (`staleTime: Infinity`), isto **muda por baixo dos pés**: o
 * usuário salva um arquivo no editor dele e o status de fora já é outro. Daí `staleTime: 0` e o
 * refetch ao voltar o foco para a aba — que é exatamente o gesto de quem alternou para o editor
 * e voltou. Um watcher de verdade (fsmonitor/inotify) é assunto de outro bloco.
 *
 * `stage`/`unstage` devolvem o `WorktreeStatus` já atualizado (Passo 48a), então a resposta
 * **substitui** o cache em vez de invalidar: um `invalidateQueries` aqui gastaria um segundo
 * round-trip para descobrir o que o servidor acabou de dizer.
 */

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { create } from "zustand";

import { api } from "./api";
import { repoKey } from "./repo";
import type {
  ApplyHunksRequest,
  CommitDone,
  CommitRequest,
  CommitTemplate,
  DiffSide,
  DiscardHunksRequest,
  FileDiff,
  StatusEntry,
  WorktreeStatus,
} from "./api-types";

export type { DiffSide } from "./api-types";

/** Os três grupos do status. É a única coisa que distingue duas linhas de mesmo caminho. */
export type StatusGroup = "staged" | "unstaged" | "untracked";

/**
 * A linha destacada no painel de status.
 *
 * Caminho **e** grupo: um arquivo parcialmente stageado (`MM`) aparece em `staged` e em
 * `unstaged` ao mesmo tempo, e as duas linhas são coisas diferentes — uma é o que vai no
 * commit, a outra é o que ficou de fora. Store à parte da seleção de commit pelo mesmo motivo
 * que aquela é à parte da de repositório: são dimensões independentes da UI.
 */
interface StatusSelection {
  path: string | null;
  group: StatusGroup | null;
  select: (path: string, group: StatusGroup) => void;
  clear: () => void;
}

export const useStatusSelection = create<StatusSelection>()((set) => ({
  path: null,
  group: null,
  select: (path, group) => set({ path, group }),
  clear: () => set({ path: null, group: null }),
}));

export function statusKey(repoId: string) {
  return ["status", repoId] as const;
}

export function useStatus(repoId: string) {
  return useQuery({
    queryKey: statusKey(repoId),
    queryFn: () => api.get<WorktreeStatus>(`/repos/${repoId}/status`),
    staleTime: 0,
  });
}

/**
 * O que o otimista faz enquanto o `git` não responde: tira os caminhos de onde estavam e os
 * põe do outro lado.
 *
 * Não tenta adivinhar o `kind` resultante — mantém o que a entrada já tinha, com a exceção de
 * `untracked` virando `added` ao ser stageado, que é a única troca que o git sempre faz e a
 * única que a UI mostraria errada de forma visível. Tudo isto é descartado assim que a resposta
 * de verdade chega; existe só para o clique não parecer travado.
 */
function predict(status: WorktreeStatus, paths: string[], to: "staged" | "unstaged"): WorktreeStatus {
  const wanted = new Set(paths);
  const from: StatusGroup[] = to === "staged" ? ["unstaged", "untracked"] : ["staged"];

  const moving: StatusEntry[] = [];
  const next: WorktreeStatus = { ...status, staged: [...status.staged], unstaged: [...status.unstaged], untracked: [...status.untracked] };

  for (const group of from) {
    for (const entry of next[group]) {
      if (wanted.has(entry.path)) {
        moving.push(
          group === "untracked" && to === "staged" ? { ...entry, kind: "added" } : entry,
        );
      }
    }
    next[group] = next[group].filter((entry) => !wanted.has(entry.path));
  }

  // Um caminho que já estava no destino não entra duas vezes.
  const already = new Set(next[to].map((entry) => entry.path));
  next[to] = [...next[to], ...moving.filter((entry) => !already.has(entry.path))].sort((a, b) =>
    a.path.localeCompare(b.path),
  );

  return next;
}

function useStagingMutation(repoId: string, route: "stage" | "unstage") {
  const queryClient = useQueryClient();
  const to = route === "stage" ? "staged" : "unstaged";

  return useMutation({
    mutationFn: (paths: string[]) =>
      api.post<WorktreeStatus>(`/repos/${repoId}/${route}`, { paths }),
    onMutate: async (paths) => {
      // Sem isto, um refetch em voo pode aterrissar depois do otimista e desfazê-lo na tela.
      await queryClient.cancelQueries({ queryKey: statusKey(repoId) });

      const previous = queryClient.getQueryData<WorktreeStatus>(statusKey(repoId));
      if (previous) {
        queryClient.setQueryData(statusKey(repoId), predict(previous, paths, to));
      }

      return { previous };
    },
    onError: (_error, _paths, context) => {
      // Falhou: o estado otimista é mentira, e o anterior é a última verdade que tivemos.
      if (context?.previous) queryClient.setQueryData(statusKey(repoId), context.previous);
    },
    onSuccess: (status) => {
      queryClient.setQueryData(statusKey(repoId), status);
      // O diff aberto na tela é de um lado que acabou de mudar — o arquivo saiu do worktree e
      // entrou no índice, ou o contrário. Aqui é invalidação de verdade: o servidor devolveu o
      // status novo, mas não os patches, e mostrar o de antes seria mostrar o lado errado.
      queryClient.invalidateQueries({ queryKey: ["worktree-diff", repoId] });
    },
  });
}

/** `git add -- <paths>`. Cobre modificado, novo e deletado no mesmo comando. */
export function useStage(repoId: string) {
  return useStagingMutation(repoId, "stage");
}

/** `git reset -- <paths>`. Funciona mesmo em repositório sem commit nenhum. */
export function useUnstage(repoId: string) {
  return useStagingMutation(repoId, "unstage");
}

/** O grupo em que a linha está já diz de que lado olhar: só `staged` está dentro do índice. */
export function sideOf(group: StatusGroup): DiffSide {
  return group === "staged" ? "staged" : "unstaged";
}

/**
 * Move trechos de um lado para o outro: `side: "unstaged"` prepara, `side: "staged"` desfaz.
 *
 * Mesmo desfecho das mutações de arquivo inteiro — o servidor devolve o `WorktreeStatus` novo,
 * que substitui o cache —, mas **sem otimista**: prever o efeito de um hunk sobre os três
 * grupos exigiria simular o patch no cliente, que é exatamente o que o `BLOCO-E.md` diz para
 * não fazer. O ida-e-volta é local e rápido.
 */
export function useApplyHunks(repoId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: ApplyHunksRequest) =>
      api.post<WorktreeStatus>(`/repos/${repoId}/apply`, request),
    onSuccess: (status) => {
      queryClient.setQueryData(statusKey(repoId), status);
      // Os dois lados mudaram: o trecho saiu de um diff e entrou no outro.
      queryClient.invalidateQueries({ queryKey: ["worktree-diff", repoId] });
    },
  });
}

/**
 * Joga fora mudanças do working tree — arquivos inteiros.
 *
 * **A única mutação deste bloco que destrói trabalho sem reflog para socorrer.** Sem otimista,
 * de propósito: mostrar o resultado antes de o git confirmar, justamente aqui, seria dizer que
 * algo foi perdido antes de saber se foi. Quem decide entre restaurar e apagar é o servidor,
 * pelo `status` — o cliente só nomeia caminhos.
 */
export function useDiscard(repoId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (paths: string[]) =>
      api.post<WorktreeStatus>(`/repos/${repoId}/discard`, { paths }),
    onSuccess: (status) => {
      queryClient.setQueryData(statusKey(repoId), status);
      queryClient.invalidateQueries({ queryKey: ["worktree-diff", repoId] });
    },
  });
}

/** O mesmo, por trecho: `git apply --reverse` no arquivo em disco. */
export function useDiscardHunks(repoId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: DiscardHunksRequest) =>
      api.post<WorktreeStatus>(`/repos/${repoId}/discard/hunks`, request),
    onSuccess: (status) => {
      queryClient.setQueryData(statusKey(repoId), status);
      queryClient.invalidateQueries({ queryKey: ["worktree-diff", repoId] });
    },
  });
}

/**
 * Fecha o commit com o que está no índice.
 *
 * Invalida bem mais que as mutações anteriores, e é o esperado: um commit novo muda o log (tem
 * uma linha a mais no topo), as refs (a branch atual andou) e o status (o que estava preparado
 * saiu). O índice de busca é reconstruído pelo job de indexação na próxima abertura — commit
 * pela interface não o atualiza ainda.
 */
export function useCommit(repoId: string) {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (request: CommitRequest) =>
      api.post<CommitDone>(`/repos/${repoId}/commit`, request),
    onSuccess: (done) => {
      queryClient.setQueryData(statusKey(repoId), done.status);
      queryClient.invalidateQueries({ queryKey: ["worktree-diff", repoId] });
      queryClient.invalidateQueries({ queryKey: ["log", repoId] });
      queryClient.invalidateQueries({ queryKey: ["refs", repoId] });
      // O `HEAD` do repositório aponta para outro commit agora — é o que a barra de topo e o
      // título da aba leem.
      queryClient.invalidateQueries({ queryKey: repoKey(repoId) });
    },
  });
}

/**
 * O `commit.template` do usuário. `staleTime: Infinity`: é config, e config não muda enquanto
 * a aba está aberta — quem a mudou por fora reabre.
 */
export function useCommitTemplate(repoId: string) {
  return useQuery({
    queryKey: ["commit-template", repoId],
    queryFn: () => api.get<CommitTemplate>(`/repos/${repoId}/commit/template`),
    staleTime: Infinity,
  });
}

/**
 * O diff de um arquivo do trabalho local.
 *
 * `staleTime: 0` pelo mesmo motivo do `useStatus`, e a chave carrega o lado: o mesmo arquivo
 * tem dois diffs diferentes ao mesmo tempo (o que está no índice e o que ficou de fora), e
 * guardar os dois sob a mesma chave mostraria um pelo outro.
 */
export function useWorktreeDiff(repoId: string, side: DiffSide, path: string | null) {
  return useQuery({
    queryKey: ["worktree-diff", repoId, side, path],
    queryFn: () =>
      api.get<FileDiff>(
        `/repos/${repoId}/diff?side=${side}&path=${encodeURIComponent(path as string)}`,
      ),
    enabled: path !== null,
    staleTime: 0,
  });
}
