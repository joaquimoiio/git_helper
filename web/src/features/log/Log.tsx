/* Orquestra o log: uma caixa no topo, e por baixo dela o log de verdade (`CommitList`, com
 * grafo) ou um dos três filtros — mensagem/autor (`SearchResults`), caminho (`PathFilter`) ou
 * conteúdo (`PickaxeFilter`) — nunca dois ao mesmo tempo, e nunca numa janela separada.
 *
 * Os três modos são botões dentro da mesma faixa, não uma segunda linha: o painel já é
 * estreito, e uma faixa a mais só para trocar de modo custaria espaço que o log precisa mais.
 */

import { useState } from "react";

import { SEARCH_DEBOUNCE_MS, useDebouncedValue } from "../../lib/search";
import { CommitList } from "./CommitList";
import { PathFilter } from "./PathFilter";
import { PickaxeFilter } from "./PickaxeFilter";
import { SearchResults } from "./SearchResults";

type Mode = "message" | "path" | "content";

const MODE_LABEL: Record<Mode, string> = {
  message: "mensagem",
  path: "caminho",
  content: "conteúdo",
};

function ModeButton({
  active,
  onClick,
  children,
}: {
  active: boolean;
  onClick: () => void;
  children: string;
}) {
  return (
    <button
      type="button"
      onClick={onClick}
      className={`text-xs px-1.5 py-0.5 rounded-sm border shrink-0 transition-colors duration-(--duration-fast) ${
        active ? "border-n-7 bg-n-4 text-n-11" : "border-n-6 text-n-9 hover:text-n-11"
      }`}
    >
      {children}
    </button>
  );
}

function ModeSwitch({ mode, onChange }: { mode: Mode; onChange: (mode: Mode) => void }) {
  return (
    <>
      {(Object.keys(MODE_LABEL) as Mode[]).map((option) => (
        <ModeButton key={option} active={mode === option} onClick={() => onChange(option)}>
          {MODE_LABEL[option]}
        </ModeButton>
      ))}
    </>
  );
}

function SearchBox({ value, onChange }: { value: string; onChange: (value: string) => void }) {
  return (
    <input
      type="text"
      value={value}
      onChange={(event) => onChange(event.target.value)}
      placeholder="buscar, ou colar um hash — autor: depois: antes:"
      className="flex-1 min-w-0 bg-transparent text-sm text-n-11 placeholder:text-n-8 outline-none"
    />
  );
}

export function Log({ repoId }: { repoId: string }) {
  const [mode, setMode] = useState<Mode>("message");
  const [query, setQuery] = useState("");
  const debounced = useDebouncedValue(query, SEARCH_DEBOUNCE_MS);
  const searching = debounced.trim() !== "";

  if (mode === "path") {
    return (
      <div className="flex-1 min-w-0 min-h-0 flex flex-col">
        <div className="flex items-center gap-2 px-3 py-1 border-b border-n-5 shrink-0">
          <ModeSwitch mode={mode} onChange={setMode} />
        </div>
        <PathFilter repoId={repoId} />
      </div>
    );
  }

  if (mode === "content") {
    return (
      <div className="flex-1 min-w-0 min-h-0 flex flex-col">
        <div className="flex items-center gap-2 px-3 py-1 border-b border-n-5 shrink-0">
          <ModeSwitch mode={mode} onChange={setMode} />
        </div>
        <PickaxeFilter repoId={repoId} />
      </div>
    );
  }

  return (
    <div className="flex-1 min-w-0 min-h-0 flex flex-col">
      <div className="flex items-center gap-2 px-3 h-8 border-b border-n-5 shrink-0">
        <ModeSwitch mode={mode} onChange={setMode} />
        <SearchBox value={query} onChange={setQuery} />
        {query !== "" && (
          <button
            type="button"
            onClick={() => setQuery("")}
            className="text-xs text-n-9 hover:text-n-11 shrink-0 transition-colors duration-(--duration-fast)"
          >
            limpar
          </button>
        )}
      </div>
      {searching ? (
        <SearchResults repoId={repoId} query={debounced} />
      ) : (
        <CommitList repoId={repoId} />
      )}
    </div>
  );
}
