/* Busca por conteúdo — pickaxe: `-S` (contagem mudou) ou `-G` (linha do diff casa um regex),
 * cancelada e reiniciada a cada tecla, com os resultados aparecendo conforme chegam.
 *
 * Diferente do filtro por caminho (Passo 45), aqui a lista cresce **enquanto** o job roda —
 * é o aceite deste passo. O progresso/cancelar em si já vêm de graça do `JobsPanel` genérico
 * (Bloco C); o que este componente faz de próprio é reiniciar o job a cada tecla e ler a
 * cauda ao vivo em vez de esperar o fim.
 */

import { useEffect, useRef, useState } from "react";

import { useCancelJob } from "../../lib/jobs";
import { pickaxeHits, usePickaxeJob, useStartPickaxe, type PickaxeMode } from "../../lib/pickaxe";
import { SEARCH_DEBOUNCE_MS, useDebouncedValue } from "../../lib/search";
import { FlatCommitList } from "./FlatCommitList";

export function PickaxeFilter({ repoId }: { repoId: string }) {
  const [mode, setMode] = useState<PickaxeMode>("string");
  const [value, setValue] = useState("");
  const debouncedValue = useDebouncedValue(value, SEARCH_DEBOUNCE_MS);

  const [jobId, setJobId] = useState<string | null>(null);
  // A busca em andamento, para cancelar antes de começar a próxima — `useState` não serviria
  // aqui porque o cancelamento precisa do valor de **antes** deste efeito rodar, não do que
  // ele está prestes a definir.
  const runningJobId = useRef<string | null>(null);

  const start = useStartPickaxe(repoId);
  const cancel = useCancelJob();
  // Referências sempre atualizadas para as mutações: `start`/`cancel` são objetos novos a
  // cada render, mas o efeito de baixo só deve reiniciar a busca quando a query ou o modo
  // mudam — não a cada render qualquer. Chamar através da ref pega a versão mais recente sem
  // precisar disso entrar na lista de dependências; a atualização em si mora num efeito
  // próprio (sem lista de dependências, roda a cada render) porque escrever numa ref durante
  // a renderização é o que o React pede para evitar.
  const startRef = useRef(start);
  const cancelRef = useRef(cancel);
  useEffect(() => {
    startRef.current = start;
    cancelRef.current = cancel;
  });

  // Query vazia é "sem busca" o tempo todo, derivado aqui — não é o efeito abaixo que zera o
  // `jobId` nesse caso, porque isso seria `setState` síncrono só para repetir o que a própria
  // entrada já diz.
  const trimmedValue = debouncedValue.trim();
  const activeJobId = trimmedValue === "" ? null : jobId;

  const job = usePickaxeJob(activeJobId);
  const hits = pickaxeHits(job);
  const running = job?.state === "running";

  useEffect(() => {
    // Cada tecla cancela a busca anterior antes de começar outra: só a mais recente
    // interessa, e duas rodando ao mesmo tempo gastariam CPU do usuário com uma resposta que
    // ele nunca vai ver.
    if (runningJobId.current) {
      cancelRef.current.mutate(runningJobId.current);
      runningJobId.current = null;
    }

    if (trimmedValue === "") {
      return;
    }

    startRef.current.mutate(
      { mode, value: trimmedValue },
      {
        onSuccess: (accepted) => {
          runningJobId.current = accepted.jobId;
          setJobId(accepted.jobId);
        },
      },
    );
  }, [trimmedValue, mode]);

  return (
    <div className="flex-1 min-w-0 min-h-0 flex flex-col">
      <div className="flex items-center gap-2 px-3 h-8 border-b border-n-5 shrink-0">
        <input
          type="text"
          value={value}
          onChange={(event) => setValue(event.target.value)}
          placeholder={mode === "string" ? "texto que apareceu ou sumiu…" : "expressão regular…"}
          className="flex-1 min-w-0 bg-transparent text-sm text-n-11 placeholder:text-n-8 outline-none"
        />
        <button
          type="button"
          onClick={() => setMode(mode === "string" ? "regex" : "string")}
          title={mode === "string" ? "-S: contagem da string mudou" : "-G: linha do diff casa o regex"}
          className="text-xs px-1.5 py-0.5 rounded-sm border border-n-6 text-n-9 hover:text-n-11 shrink-0 transition-colors duration-(--duration-fast)"
        >
          {mode === "string" ? "-S" : "-G"}
        </button>
      </div>

      {value.trim() === "" ? (
        <p className="px-3 py-2 text-sm text-n-9">
          digite um trecho de código ou texto para achar onde ele apareceu ou sumiu.
        </p>
      ) : job?.state === "error" ? (
        <p className="px-3 py-2 text-sm text-error">{job.message}</p>
      ) : (
        <>
          {running && <p className="px-3 py-1 text-xs text-n-8 shrink-0">buscando…</p>}
          {hits.length === 0 ? (
            <p className="px-3 py-2 text-sm text-n-9">
              {running ? "procurando os primeiros resultados…" : "nenhum commit encontrado."}
            </p>
          ) : (
            <FlatCommitList hits={hits} ariaLabel="commits que tocam o conteúdo buscado" />
          )}
        </>
      )}
    </div>
  );
}
