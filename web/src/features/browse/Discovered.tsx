/* Repositórios encontrados debaixo da raiz configurada.
 *
 * Complementa os recentes: recentes é "o que eu estava fazendo", isto é "o que existe nesta
 * máquina". Quem acabou de instalar não tem recente nenhum, e esta lista é o que evita mandá-lo
 * navegar a home pasta por pasta.
 *
 * A raiz e a profundidade saem do `config.toml` — a UI só mostra qual raiz foi usada, para o
 * usuário entender por que o repositório dele não está aqui.
 */

import { useQueryClient } from "@tanstack/react-query";

import type { ScanEntry } from "../../lib/api-types";
import { useDiscovered } from "../../lib/repo";

export interface DiscoveredProps {
  onPick: (entry: ScanEntry) => void;
}

export function Discovered({ onPick }: DiscoveredProps) {
  const scan = useDiscovered();
  const queryClient = useQueryClient();

  if (scan.isPending || !scan.data || scan.data.repos.length === 0) return null;

  const { root, repos, truncated } = scan.data;

  return (
    <section className="shrink-0 border-b border-n-5">
      <div className="flex items-baseline gap-2 px-3 pt-2 pb-1">
        <h2 className="text-xs uppercase tracking-[0.1em] text-n-8">nesta máquina</h2>
        <span className="text-xs font-mono text-n-8 truncate">{root}</span>
        <button
          type="button"
          onClick={() => queryClient.invalidateQueries({ queryKey: ["scan"] })}
          disabled={scan.isFetching}
          className="ml-auto text-xs text-n-8 hover:text-n-11 disabled:text-n-7 transition-colors duration-(--duration-fast)"
        >
          {scan.isFetching ? "varrendo…" : "recarregar"}
        </button>
      </div>

      <ul className="max-h-48 overflow-y-auto">
        {repos.map((entry) => (
          <li key={entry.path}>
            <button
              type="button"
              onClick={() => onPick(entry)}
              className="flex items-baseline gap-2 w-full px-3 py-0.5 text-left hover:bg-n-3 transition-colors duration-(--duration-fast)"
            >
              <span className="text-sm text-n-8 shrink-0 w-3">◆</span>
              <span className="text-base text-n-11 truncate">{entry.name}</span>
              {/* O caminho relativo só acrescenta informação quando não é o próprio nome. */}
              {entry.relative !== entry.name && (
                <span className="text-xs font-mono text-n-8 truncate">{entry.relative}</span>
              )}
            </button>
          </li>
        ))}
      </ul>

      {truncated && (
        <p className="px-3 pb-1 text-xs text-n-8">
          a varredura parou no limite — o que falta está no navegador de pastas abaixo.
        </p>
      )}
    </section>
  );
}
