/* Filtro por caminho: um campo com autocomplete que dispara o job `path-filter` (Passo 45a) e
 * mostra o resultado como lista achatada — sem grafo, pelo mesmo motivo da busca (Passo 43).
 *
 * O progresso e o cancelar já existem: o `JobsPanel` (Bloco C) mostra qualquer job pelo
 * `kind`, e este é só mais um. O que falta aqui é só a caixa e a lista de resultado.
 */

import { useState } from "react";

import { useCancelJob } from "../../lib/jobs";
import { pathFilterHits, usePathAutocomplete, usePathFilterJob, useStartPathFilter } from "../../lib/pathFilter";
import { SEARCH_DEBOUNCE_MS, useDebouncedValue } from "../../lib/search";
import { FlatCommitList } from "./FlatCommitList";

export function PathFilter({ repoId }: { repoId: string }) {
  const [input, setInput] = useState("");
  const [suggestionsOpen, setSuggestionsOpen] = useState(false);
  const [jobId, setJobId] = useState<string | null>(null);

  const debouncedInput = useDebouncedValue(input, SEARCH_DEBOUNCE_MS);
  const suggestions = usePathAutocomplete(repoId, debouncedInput);
  const start = useStartPathFilter(repoId);
  const cancel = useCancelJob();
  const job = usePathFilterJob(jobId);
  const hits = pathFilterHits(job);

  function runFilter(path: string) {
    if (path === "") return;
    setInput(path);
    setSuggestionsOpen(false);
    start.mutate(path, { onSuccess: (accepted) => setJobId(accepted.jobId) });
  }

  function onKeyDown(event: React.KeyboardEvent) {
    if (event.key === "Enter") {
      event.preventDefault();
      runFilter(input);
      return;
    }

    // Completa para a primeira sugestão. Se ela for uma pasta (termina em `/`), o campo muda
    // e a busca de sugestões roda de novo para o nível de dentro — sem código extra para isso.
    if (event.key === "Tab" && suggestions.data?.[0]) {
      event.preventDefault();
      setInput(suggestions.data[0]);
    }
  }

  return (
    <div className="flex-1 min-w-0 min-h-0 flex flex-col">
      <div className="relative border-b border-n-5 shrink-0">
        <input
          type="text"
          value={input}
          onChange={(event) => {
            setInput(event.target.value);
            setSuggestionsOpen(true);
            setJobId(null);
          }}
          onFocus={() => setSuggestionsOpen(true)}
          onKeyDown={onKeyDown}
          placeholder="caminho do arquivo — Tab completa, Enter filtra"
          className="w-full h-8 px-3 bg-transparent text-sm text-n-11 placeholder:text-n-8 outline-none"
        />

        {suggestionsOpen && (suggestions.data?.length ?? 0) > 0 && (
          <ul className="absolute inset-x-0 top-full z-10 max-h-48 overflow-y-auto border-b border-n-5 bg-n-1 shadow-sm">
            {suggestions.data?.map((path) => (
              <li
                key={path}
                onClick={() => runFilter(path)}
                className="px-3 py-1 text-sm font-mono text-n-10 truncate cursor-default hover:bg-n-3 transition-colors duration-(--duration-fast)"
              >
                {path}
              </li>
            ))}
          </ul>
        )}
      </div>

      {job === undefined ? (
        <p className="px-3 py-2 text-sm text-n-9">
          digite um caminho e pressione enter para ver só os commits que o tocam.
        </p>
      ) : job.state === "running" ? (
        <div className="flex items-center justify-between px-3 py-2 text-sm text-n-8">
          <span>filtrando…</span>
          <button
            type="button"
            onClick={() => cancel.mutate(job.jobId)}
            disabled={cancel.isPending}
            className="text-xs text-n-9 hover:text-n-11 disabled:text-n-7 transition-colors duration-(--duration-fast)"
          >
            cancelar
          </button>
        </div>
      ) : job.state === "error" ? (
        <p className="px-3 py-2 text-sm text-error">{job.message}</p>
      ) : job.state === "cancelled" ? (
        <p className="px-3 py-2 text-sm text-n-9">busca cancelada.</p>
      ) : hits.length === 0 ? (
        <p className="px-3 py-2 text-sm text-n-9">nenhum commit tocou este arquivo.</p>
      ) : (
        <FlatCommitList hits={hits} ariaLabel="commits que tocam o arquivo" />
      )}
    </div>
  );
}
