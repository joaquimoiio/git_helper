/* Operações em andamento.
 *
 * Fica na faixa de baixo, acima da barra de status, e só aparece quando há algo acontecendo: o
 * conteúdo é a interface, e uma área permanentemente vazia reservada para "talvez um dia haja um
 * job" é espaço tirado do log.
 *
 * A barra de progresso aceita `fraction: null` — é o estado mais comum no começo de um clone,
 * quando o git ainda não sabe quantos objetos vêm. Uma barra que finge saber é pior que uma
 * listra indeterminada.
 */

import { useEffect, useState } from "react";

import type { JobSnapshot } from "../../lib/api-types";
import { useCancelJob, useJobs } from "../../lib/jobs";

/** Terminados ficam à vista por um tempo, para o usuário ver o que aconteceu e não sumir. */
const KEEP_FINISHED_MS = 30_000;

const LABEL: Record<JobSnapshot["state"], string> = {
  running: "em andamento",
  done: "concluído",
  error: "falhou",
  cancelled: "cancelado",
};

function Bar({ fraction }: { fraction: number | null }) {
  return (
    <div className="h-0.5 w-32 shrink-0 bg-n-4 overflow-hidden">
      <div
        className={`h-full bg-n-9 transition-[width] duration-(--duration-slow) ${
          fraction === null ? "w-1/3 opacity-60" : ""
        }`}
        style={fraction === null ? undefined : { width: `${Math.round(fraction * 100)}%` }}
      />
    </div>
  );
}

function Row({ job }: { job: JobSnapshot }) {
  const cancel = useCancelJob();
  const [details, setDetails] = useState(false);
  const running = job.state === "running";

  return (
    <li className="border-t border-n-5">
      <div className="flex items-center gap-3 px-3 py-1">
        <span className="text-sm font-mono text-n-9 shrink-0">{job.kind}</span>

        {running ? (
          <Bar fraction={job.progress?.fraction ?? null} />
        ) : (
          <span
            className={`text-sm shrink-0 ${job.state === "error" ? "text-error" : "text-n-9"}`}
          >
            {LABEL[job.state]}
          </span>
        )}

        <span className="text-sm text-n-10 truncate">
          {job.message ?? job.progress?.detail ?? job.progress?.phase ?? job.log.at(-1) ?? ""}
        </span>

        <div className="ml-auto flex items-center gap-3 shrink-0">
          {/* O stderr cru fica **atrás** disto, nunca como a mensagem principal. */}
          {job.log.length > 0 && (
            <button
              type="button"
              onClick={() => setDetails((open) => !open)}
              className="text-xs text-n-9 hover:text-n-11 transition-colors duration-(--duration-fast)"
            >
              {details ? "ocultar detalhes" : "ver detalhes"}
            </button>
          )}
          {running && (
            <button
              type="button"
              onClick={() => cancel.mutate(job.jobId)}
              disabled={cancel.isPending}
              className="text-xs text-n-9 hover:text-n-11 disabled:text-n-7 transition-colors duration-(--duration-fast)"
            >
              cancelar
            </button>
          )}
        </div>
      </div>

      {job.action && (
        <p className="px-3 pb-1 text-sm text-n-9">{job.action}</p>
      )}

      {details && (
        <pre className="px-3 pb-2 max-h-40 overflow-auto text-xs font-mono text-n-9 whitespace-pre-wrap">
          {job.log.join("\n")}
        </pre>
      )}
    </li>
  );
}

export function JobsPanel() {
  const jobs = useJobs();
  // O relógio é estado, não uma leitura no meio do render: `Date.now()` durante o render dá um
  // valor diferente a cada passada e o React não tem como saber quando ele mudou.
  const [now, setNow] = useState(Date.now);

  const all = jobs.data ?? [];
  const finished = all.some((job) => job.finishedAt !== null);

  // Só enquanto houver algo terminado à vista: sem isto o job concluído ficaria na tela até
  // algum outro evento provocar um render.
  useEffect(() => {
    if (!finished) return;

    const timer = setInterval(() => setNow(Date.now()), 5_000);
    return () => clearInterval(timer);
  }, [finished]);

  const visible = all.filter(
    (job) =>
      job.state === "running" ||
      (job.finishedAt !== null && now - job.finishedAt < KEEP_FINISHED_MS),
  );

  if (visible.length === 0) return null;

  return (
    <section className="shrink-0 bg-n-1">
      <ul>
        {visible.map((job) => (
          <Row key={job.jobId} job={job} />
        ))}
      </ul>
    </section>
  );
}
