/* O único componente do app que menciona tema — e mesmo ele só escreve a *escolha*, nunca
 * lê qual tema está pintando. Todo o resto pede token. */

import { useTheme, type ThemeChoice } from "../lib/theme";

const OPTIONS: ReadonlyArray<[ThemeChoice, string]> = [
  ["system", "auto"],
  ["dark", "escuro"],
  ["light", "claro"],
];

export function ThemeSwitch() {
  const choice = useTheme((state) => state.choice);
  const setChoice = useTheme((state) => state.setChoice);

  return (
    <div className="flex items-stretch border border-n-6 rounded-sm overflow-hidden text-xs">
      {OPTIONS.map(([option, label]) => (
        <button
          key={option}
          type="button"
          onClick={() => setChoice(option)}
          aria-pressed={choice === option}
          className={`px-2 transition-colors duration-(--duration-fast) not-first:border-l not-first:border-n-6 ${
            choice === option ? "bg-n-4 text-n-11" : "text-n-9 hover:bg-n-3"
          }`}
        >
          {label}
        </button>
      ))}
    </div>
  );
}
