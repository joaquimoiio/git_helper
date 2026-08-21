/* Estado de layout — larguras e colapso dos painéis laterais.
 *
 * Zustand com `persist`: é estado de UI, nunca vem do servidor, e o usuário espera que a
 * janela reabra do jeito que ele deixou. `localStorage` é síncrono, então a largura certa
 * já vale no primeiro render — sem painel pulando de tamanho depois da hidratação.
 */

import { create } from "zustand";
import { persist } from "zustand/middleware";

/* Limites em px. O mínimo é o ponto em que o painel ainda mostra conteúdo útil; abaixo
 * disso o certo é colapsar, não encolher. */
export const SIDEBAR = { min: 160, max: 480, initial: 232 } as const;
export const DETAIL = { min: 240, max: 720, initial: 360 } as const;

export type Panel = "sidebar" | "detail";

interface LayoutStore {
  sidebarWidth: number;
  detailWidth: number;
  sidebarCollapsed: boolean;
  detailCollapsed: boolean;
  setWidth: (panel: Panel, width: number) => void;
  toggle: (panel: Panel) => void;
}

function clamp(value: number, { min, max }: { min: number; max: number }) {
  return Math.min(max, Math.max(min, Math.round(value)));
}

export const useLayout = create<LayoutStore>()(
  persist(
    (set) => ({
      sidebarWidth: SIDEBAR.initial,
      detailWidth: DETAIL.initial,
      sidebarCollapsed: false,
      detailCollapsed: false,

      setWidth: (panel, width) =>
        set(
          panel === "sidebar"
            ? { sidebarWidth: clamp(width, SIDEBAR) }
            : { detailWidth: clamp(width, DETAIL) },
        ),

      toggle: (panel) =>
        set((state) =>
          panel === "sidebar"
            ? { sidebarCollapsed: !state.sidebarCollapsed }
            : { detailCollapsed: !state.detailCollapsed },
        ),
    }),
    {
      name: "porc.layout",
      // Versionado desde o começo: quando o layout ganhar um quarto painel, um `migrate`
      // evita que a chave antiga deixe alguém com a tela quebrada e sem saída.
      version: 1,
    },
  ),
);
