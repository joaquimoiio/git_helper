/* Qual repositório está aberto.
 *
 * O store guarda só o `repoId` — a *seleção*, que é estado de UI. Quem o repositório é
 * (branch, HEAD, nome) vem do servidor e mora no TanStack Query; duplicar isso aqui daria dois
 * lugares para a branch atual ficar desatualizada.
 *
 * Não é persistido de propósito: o registry do servidor vive em memória e morre com o
 * processo, então um id guardado no `localStorage` apontaria para nada no boot seguinte. É a
 * lista de recentes (Passo 26) que atravessa boots, e ela guarda caminho, não id.
 */

import { useMutation, useQuery, useQueryClient } from "@tanstack/react-query";
import { create } from "zustand";

import { api } from "./api";
import type { InitRequest, RecentEntry, RefMarker, Repo, Scan } from "./api-types";

interface RepoStore {
  repoId: string | null;
  setRepoId: (repoId: string | null) => void;
}

export const useRepoSelection = create<RepoStore>()((set) => ({
  repoId: null,
  setRepoId: (repoId) => set({ repoId }),
}));

export function repoKey(repoId: string) {
  return ["repo", repoId] as const;
}

/** O repositório aberto, ou `undefined` enquanto não houver um. */
export function useOpenRepo() {
  const repoId = useRepoSelection((state) => state.repoId);

  return useQuery({
    queryKey: repoKey(repoId ?? ""),
    queryFn: () => api.get<Repo>(`/repos/${repoId}`),
    enabled: repoId !== null,
  });
}

/** Abre uma pasta como repositório e a seleciona. */
export function useOpenRepoMutation() {
  const queryClient = useQueryClient();
  const setRepoId = useRepoSelection((state) => state.setRepoId);

  return useMutation({
    mutationFn: (path: string) => api.post<Repo>("/repos", { path }),
    onSuccess: (repo) => {
      // Semeia o cache com a resposta que já temos: sem isto o `useOpenRepo` dispararia um
      // GET idêntico logo depois do POST, só para descobrir o que acabou de ser dito.
      queryClient.setQueryData(repoKey(repo.repoId), repo);
      // Abrir mexe na lista de recentes do servidor — daí a invalidação e não um `setQueryData`.
      queryClient.invalidateQueries({ queryKey: RECENTS_KEY });
      setRepoId(repo.repoId);
    },
  });
}

/**
 * Cria um repositório e o seleciona.
 *
 * Mesmo `onSuccess` do abrir, porque o servidor já devolve o repositório aberto — criar e abrir
 * são um passo só, e a UI não precisa saber que houve um `git init` no meio.
 */
export function useInitRepo() {
  const queryClient = useQueryClient();
  const setRepoId = useRepoSelection((state) => state.setRepoId);

  return useMutation({
    mutationFn: (request: InitRequest) => api.post<Repo>("/repos/init", request),
    onSuccess: (repo) => {
      queryClient.setQueryData(repoKey(repo.repoId), repo);
      queryClient.invalidateQueries({ queryKey: RECENTS_KEY });
      // A varredura tem um repositório a mais agora.
      queryClient.invalidateQueries({ queryKey: ["scan"] });
      setRepoId(repo.repoId);
    },
  });
}

const RECENTS_KEY = ["recents"] as const;

/** Repositórios lembrados entre boots, do mais recente para o mais antigo. */
export function useRecents() {
  return useQuery({
    queryKey: RECENTS_KEY,
    queryFn: () => api.get<RecentEntry[]>("/recents"),
  });
}

/**
 * Repositórios debaixo da raiz configurada.
 *
 * A varredura percorre diretórios de verdade, então é a consulta mais cara da tela inicial —
 * daí o `staleTime` longo. Quem criou um repositório novo tem o botão de recarregar.
 */
export function useDiscovered() {
  return useQuery({
    queryKey: ["scan"],
    queryFn: () => api.get<Scan>("/fs/scan"),
    staleTime: 60_000,
  });
}

/**
 * Toda ponta do repositório (branches, remotas, tags, `HEAD` destacado), para marcar linhas
 * do log. `staleTime: Infinity` como o log: nada por baixo dos pés muda enquanto a aba está
 * aberta, é o WebSocket (Bloco E) que vai invalidar isto quando o usuário criar ou mover uma
 * branch por fora.
 */
export function useRefs(repoId: string) {
  return useQuery({
    queryKey: ["refs", repoId],
    queryFn: () => api.get<RefMarker[]>(`/repos/${repoId}/refs`),
    staleTime: Infinity,
  });
}

/** Tira um repositório da lista de recentes — não toca no disco do usuário. */
export function useForgetRecent() {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (repoId: string) => api.del(`/recents/${repoId}`),
    onSuccess: () => queryClient.invalidateQueries({ queryKey: RECENTS_KEY }),
  });
}
