/* Repositórios recentes, com atalho numérico.
 *
 * É a primeira coisa da tela inicial porque é a resposta certa quase sempre: quem abre o
 * porcelain de manhã quer o repositório de ontem, não um passeio pela home. As nove primeiras
 * entradas respondem às teclas 1–9 — sem modificador, porque a tela inicial não tem campo de
 * texto para o dígito atrapalhar.
 */

import { useEffect, useMemo } from "react";

import type { RecentEntry } from "../../lib/api-types";
import { useForgetRecent, useRecents } from "../../lib/repo";

/** Até onde vai o atalho de uma tecla. Além disso, o mouse ou o navegador de pastas. */
const SHORTCUTS = 9;

const RELATIVE = new Intl.RelativeTimeFormat("pt-BR", { numeric: "auto" });

const UNITS = [
  ["second", 60],
  ["minute", 60],
  ["hour", 24],
  ["day", 7],
  ["week", 4.35],
  ["month", 12],
  ["year", Infinity],
] as const;

/** "há 2 horas", "ontem". Data absoluta numa lista de nove linhas não diz nada de útil. */
function since(epochMs: number): string {
  let value = (epochMs - Date.now()) / 1000;

  for (const [unit, step] of UNITS) {
    if (Math.abs(value) < step) return RELATIVE.format(Math.round(value), unit);
    value /= step;
  }

  return RELATIVE.format(Math.round(value), "year");
}

export interface RecentsProps {
  onPick: (entry: RecentEntry) => void;
}

export function Recents({ onPick }: RecentsProps) {
  const recents = useRecents();
  const forget = useForgetRecent();

  // Memoizado por causa do `?? []`, que devolveria um array novo a cada render e faria o
  // listener global de teclado ser trocado junto.
  const entries = useMemo(() => recents.data ?? [], [recents.data]);

  useEffect(() => {
    function onKeyDown(event: KeyboardEvent) {
      if (event.ctrlKey || event.metaKey || event.altKey) return;

      const index = Number(event.key) - 1;
      if (!Number.isInteger(index) || index < 0 || index >= Math.min(SHORTCUTS, entries.length)) {
        return;
      }

      const entry = entries[index];
      if (!entry.available) return;

      event.preventDefault();
      onPick(entry);
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [entries, onPick]);

  if (entries.length === 0) return null;

  return (
    <section className="shrink-0 border-b border-n-5">
      <h2 className="px-3 pt-2 pb-1 text-xs uppercase tracking-[0.1em] text-n-8">recentes</h2>
      <ul className="max-h-64 overflow-y-auto">
        {entries.map((entry, index) => (
          <li
            key={entry.repoId}
            className="group flex items-baseline gap-2 px-3 py-0.5 hover:bg-n-3 transition-colors duration-(--duration-fast)"
          >
            <span className="w-3 shrink-0 text-xs font-mono text-n-8">
              {index < SHORTCUTS ? index + 1 : ""}
            </span>
            <button
              type="button"
              disabled={!entry.available}
              onClick={() => onPick(entry)}
              className="flex items-baseline gap-2 min-w-0 flex-1 text-left disabled:cursor-not-allowed"
            >
              <span
                className={`text-base truncate ${entry.available ? "text-n-11" : "text-n-8 line-through"}`}
              >
                {entry.name}
              </span>
              <span className="text-xs font-mono text-n-8 truncate">{entry.path}</span>
            </button>
            <span className="text-xs text-n-8 shrink-0">
              {entry.available ? since(entry.openedAt) : "sumiu do disco"}
            </span>
            {/* Só aparece no hover: esquecer é ação rara, e um botão fixo por linha faria a
                lista parecer um formulário. */}
            <button
              type="button"
              onClick={() => forget.mutate(entry.repoId)}
              className="shrink-0 text-xs text-n-8 opacity-0 group-hover:opacity-100 focus-visible:opacity-100 hover:text-n-11 transition-[color,opacity] duration-(--duration-fast)"
            >
              esquecer
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}
