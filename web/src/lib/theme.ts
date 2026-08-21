/* Estado de tema — Zustand, porque é estado de UI puro: não vem do servidor e nunca
 * volta para ele.
 *
 * O store guarda só a *escolha*, que tem três valores. `system` é ausência de escolha, e é
 * representada pela ausência do atributo `data-theme` — assim o CSS decide sozinho pelo
 * `color-scheme: dark light`, já no primeiro pixel. Nenhum componente lê qual tema está
 * ativo; quem quiser cor pede o token.
 */

import { create } from "zustand";

export type ThemeChoice = "system" | "dark" | "light";

const STORAGE_KEY = "porc.theme";

function isChoice(value: unknown): value is ThemeChoice {
  return value === "system" || value === "dark" || value === "light";
}

function read(): ThemeChoice {
  // localStorage pode lançar (modo privado, storage desabilitado). Um app de git não
  // deixa de abrir por causa da preferência de tema.
  try {
    const stored = window.localStorage.getItem(STORAGE_KEY);
    return isChoice(stored) ? stored : "system";
  } catch {
    return "system";
  }
}

function apply(choice: ThemeChoice) {
  const root = document.documentElement;
  if (choice === "system") root.removeAttribute("data-theme");
  else root.setAttribute("data-theme", choice);

  try {
    if (choice === "system") window.localStorage.removeItem(STORAGE_KEY);
    else window.localStorage.setItem(STORAGE_KEY, choice);
  } catch {
    /* preferência não persistida é degradação aceitável */
  }
}

interface ThemeStore {
  choice: ThemeChoice;
  setChoice: (choice: ThemeChoice) => void;
}

export const useTheme = create<ThemeStore>((set) => ({
  choice: read(),
  setChoice: (choice) => {
    apply(choice);
    set({ choice });
  },
}));

/** Reidrata o atributo no boot. Chamado uma vez, antes do render. */
export function bootTheme() {
  apply(useTheme.getState().choice);
}
