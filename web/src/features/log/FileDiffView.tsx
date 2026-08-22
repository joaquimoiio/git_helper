/* O diff de um arquivo: hunks estruturados, modo unificado ou lado a lado, realce leve
 * (fundo do token de adição/remoção — sem realce de sintaxe, que é biblioteca e peso que o
 * bloco não pediu).
 *
 * Binário e conteúdo não-UTF8 têm aviso próprio, nunca tentam desenhar um hunk que não existe.
 */

import { useState } from "react";

import type { DiffHunk, DiffLine, FileDiff, HunkPick } from "../../lib/api-types";
import { useFileDiff } from "../../lib/commit";

interface SideBySideRow {
  left: DiffLine | null;
  right: DiffLine | null;
}

/**
 * Pareia remoção com adição, índice a índice, dentro de cada corrida entre linhas de
 * contexto — o padrão normal de um hunk. Não é alinhamento por conteúdo (LCS): o lado mais
 * curto de uma corrida desigual fica com célula em branco, não uma linha "parecida".
 */
function toSideBySideRows(lines: DiffLine[]): SideBySideRow[] {
  const rows: SideBySideRow[] = [];
  let i = 0;

  while (i < lines.length) {
    const line = lines[i];
    if (line.kind === "context") {
      rows.push({ left: line, right: line });
      i++;
      continue;
    }

    const deletions: DiffLine[] = [];
    while (i < lines.length && lines[i].kind === "deletion") {
      deletions.push(lines[i]);
      i++;
    }
    const additions: DiffLine[] = [];
    while (i < lines.length && lines[i].kind === "addition") {
      additions.push(lines[i]);
      i++;
    }

    const rowCount = Math.max(deletions.length, additions.length);
    for (let k = 0; k < rowCount; k++) {
      rows.push({ left: deletions[k] ?? null, right: additions[k] ?? null });
    }
  }

  return rows;
}

function lineStyle(kind: DiffLine["kind"]) {
  if (kind === "addition") return { bg: "bg-add-bg", fg: "text-add" };
  if (kind === "deletion") return { bg: "bg-del-bg", fg: "text-del" };
  return { bg: "", fg: "text-n-10" };
}

/**
 * O que dá para fazer com **um** hunk, quando há o que fazer.
 *
 * Ausente é o caso do diff de um commit: um commit é imutável, e um botão ali não teria para
 * onde apontar. Presente é o trabalho local, onde cada hunk pode atravessar para o outro lado.
 */
export interface HunkAction {
  label: string;
  onApply: (picks: HunkPick[]) => void;
  pending: boolean;
}

/**
 * A numeração que o servidor entende: posição **entre as linhas de mudança** do hunk.
 *
 * Contexto não conta, e não conta dos dois lados do fio — é o mesmo critério do
 * `porc-git::patch`. Numerar sobre todas as linhas divergiria nos arquivos sem quebra de linha
 * no fim, onde o texto do patch tem uma linha a mais que a lista desenhada aqui.
 */
function changeIndexes(lines: DiffLine[]): (number | null)[] {
  let next = 0;
  return lines.map((line) => (line.kind === "context" ? null : next++));
}

function HunkHeader({
  header,
  index,
  action,
  destructive,
  picked,
  onClear,
}: {
  header: string;
  index: number;
  action?: HunkAction;
  destructive?: HunkAction;
  picked: number[];
  onClear: () => void;
}) {
  const parcial = picked.length > 0;

  return (
    <div className="flex items-baseline gap-2 px-2 py-0.5 bg-n-2 text-n-8">
      <span className="whitespace-pre truncate flex-1 min-w-0">{header}</span>
      {action && parcial && (
        <button
          type="button"
          onClick={onClear}
          className="shrink-0 text-xs px-1.5 text-n-9 hover:text-n-11 transition-colors duration-(--duration-fast)"
        >
          limpar
        </button>
      )}
      {action && (
        <button
          type="button"
          onClick={() =>
            action.onApply([parcial ? { hunk: index, lines: picked } : { hunk: index }])
          }
          disabled={action.pending}
          className="shrink-0 text-xs px-1.5 rounded-sm border border-n-6 text-n-9 hover:text-n-11 disabled:text-n-7 transition-colors duration-(--duration-fast)"
        >
          {parcial ? `${action.label} ${picked.length} linha${picked.length > 1 ? "s" : ""}` : action.label}
        </button>
      )}
      {/* O botão destrutivo vem por último e com a cor de remoção: é o mais longe do gesto
          normal, e o único cuja cor diz que ele é diferente dos outros. */}
      {destructive && (
        <button
          type="button"
          onClick={() =>
            destructive.onApply([parcial ? { hunk: index, lines: picked } : { hunk: index }])
          }
          disabled={destructive.pending}
          className="shrink-0 text-xs px-1.5 rounded-sm border border-n-6 text-del hover:bg-del-bg disabled:text-n-7 transition-colors duration-(--duration-fast)"
        >
          {destructive.label}
        </button>
      )}
    </div>
  );
}

function UnifiedLine({
  line,
  selectable,
  selected,
  onToggle,
}: {
  line: DiffLine;
  selectable: boolean;
  selected: boolean;
  onToggle: () => void;
}) {
  const { bg, fg } = lineStyle(line.kind);
  const marker = line.kind === "addition" ? "+" : line.kind === "deletion" ? "-" : " ";

  return (
    <div
      onClick={selectable ? onToggle : undefined}
      className={`flex whitespace-pre ${selected ? "bg-n-4" : bg} ${
        selectable ? "cursor-default hover:bg-n-3" : ""
      }`}
    >
      {/* Coluna da marca: existe sempre que houver ação, para as linhas não dançarem de lugar
          quando uma é escolhida. */}
      {selectable && (
        <span className="w-3 shrink-0 text-center text-n-9 select-none">{selected ? "•" : ""}</span>
      )}
      <span className="w-9 shrink-0 text-right pr-1 text-n-8 select-none">{line.oldLineno ?? ""}</span>
      <span className="w-9 shrink-0 text-right pr-1 text-n-8 select-none">{line.newLineno ?? ""}</span>
      <span className={`pl-1 ${fg}`}>
        {marker}
        {line.content}
      </span>
    </div>
  );
}

function UnifiedView({
  hunks,
  action,
  destructive,
  picks,
  onToggleLine,
  onClearHunk,
}: {
  hunks: DiffHunk[];
  action?: HunkAction;
  destructive?: HunkAction;
  picks: ReadonlyMap<number, number[]>;
  onToggleLine: (hunk: number, line: number) => void;
  onClearHunk: (hunk: number) => void;
}) {
  return (
    <div className="overflow-x-auto font-mono text-xs">
      {hunks.map((hunk, hunkIndex) => {
        const indexes = changeIndexes(hunk.lines);
        const picked = picks.get(hunkIndex) ?? [];

        return (
          <div key={hunkIndex}>
            <HunkHeader
              header={hunk.header}
              index={hunkIndex}
              action={action}
              destructive={destructive}
              picked={picked}
              onClear={() => onClearHunk(hunkIndex)}
            />
            {hunk.lines.map((line, lineIndex) => {
              const change = indexes[lineIndex];

              return (
                // A posição na lista é a chave estável: hunks não reordenam linhas entre eles.
                <UnifiedLine
                  key={lineIndex}
                  line={line}
                  selectable={action !== undefined && change !== null}
                  selected={change !== null && picked.includes(change)}
                  onToggle={() => change !== null && onToggleLine(hunkIndex, change)}
                />
              );
            })}
          </div>
        );
      })}
    </div>
  );
}

function SideCell({ line }: { line: DiffLine | null }) {
  if (!line) return <div className="flex-1 min-w-0 bg-n-2" />;

  const { bg, fg } = lineStyle(line.kind);
  const lineno = line.oldLineno ?? line.newLineno;

  return (
    <div className={`flex flex-1 min-w-0 whitespace-pre ${bg}`}>
      <span className="w-9 shrink-0 text-right pr-1 text-n-8 select-none">{lineno}</span>
      <span className={`pl-1 ${fg}`}>{line.content}</span>
    </div>
  );
}

/* Lado a lado **não** tem seleção por linha: uma linha da esquerda e uma da direita dividem a
 * mesma fileira, e um clique ali seria ambíguo entre "esta remoção" e "esta adição". Quem quer
 * escolher linha troca para o unificado, que é onde cada linha é uma linha. */
function SideBySideView({
  hunks,
  action,
  destructive,
}: {
  hunks: DiffHunk[];
  action?: HunkAction;
  destructive?: HunkAction;
}) {
  return (
    <div className="overflow-x-auto font-mono text-xs">
      {hunks.map((hunk, hunkIndex) => (
        <div key={hunkIndex}>
          <HunkHeader
            header={hunk.header}
            index={hunkIndex}
            action={action}
            destructive={destructive}
            picked={[]}
            onClear={() => {}}
          />
          {toSideBySideRows(hunk.lines).map((row, rowIndex) => (
            <div key={rowIndex} className="flex">
              <SideCell line={row.left} />
              <div className="w-px shrink-0 bg-n-5" />
              <SideCell line={row.right} />
            </div>
          ))}
        </div>
      ))}
    </div>
  );
}

function ModeToggle({
  mode,
  onChange,
}: {
  mode: "unified" | "side-by-side";
  onChange: (mode: "unified" | "side-by-side") => void;
}) {
  const option = (value: "unified" | "side-by-side", label: string) => (
    <button
      type="button"
      onClick={() => onChange(value)}
      className={`text-xs px-1.5 py-0.5 rounded-sm border transition-colors duration-(--duration-fast) ${
        mode === value ? "border-n-7 bg-n-4 text-n-11" : "border-n-6 text-n-9 hover:text-n-11"
      }`}
    >
      {label}
    </button>
  );

  return (
    <div className="flex items-center gap-1 px-3 py-1 border-b border-n-5">
      {option("unified", "unificado")}
      {option("side-by-side", "lado a lado")}
    </div>
  );
}

/**
 * O visualizador em si, sem saber de onde o patch veio.
 *
 * É o que o Passo 49 reaproveita: o diff de um commit e o diff do trabalho local têm a mesma
 * forma (`FileDiff`) de propósito, e um segundo componente com as mesmas três formas e os
 * mesmos dois modos seria duas versões da mesma regra de apresentação.
 */
export function DiffView({
  diff,
  action,
  destructive,
}: {
  diff: FileDiff;
  action?: HunkAction;
  /** A ação que **perde** trabalho (descartar). Separada da normal para ter cor e lugar
   *  próprios — e para quem a passa ser obrigado a pensar duas vezes antes de passá-la. */
  destructive?: HunkAction;
}) {
  const [mode, setMode] = useState<"unified" | "side-by-side">("unified");
  // Hunk → linhas escolhidas dentro dele. Some quando o componente é remontado, e quem garante
  // isso ao trocar de arquivo é a `key` de quem chama: uma seleção de linhas do arquivo
  // anterior aplicada ao seguinte seria um patch em cima do arquivo errado.
  const [picks, setPicks] = useState<ReadonlyMap<number, number[]>>(new Map());

  function toggleLine(hunk: number, line: number) {
    setPicks((previous) => {
      const next = new Map(previous);
      const current = next.get(hunk) ?? [];
      const without = current.filter((item) => item !== line);

      if (without.length === current.length) next.set(hunk, [...current, line].sort((a, b) => a - b));
      else if (without.length === 0) next.delete(hunk);
      else next.set(hunk, without);

      return next;
    });
  }

  function clearHunk(hunk: number) {
    setPicks((previous) => {
      const next = new Map(previous);
      next.delete(hunk);
      return next;
    });
  }

  if (diff.kind === "binary") {
    return <p className="px-3 py-2 text-sm text-n-9">arquivo binário — sem diff de texto.</p>;
  }

  if (diff.kind === "notUtf8") {
    return (
      <p className="px-3 py-2 text-sm text-n-9">
        este arquivo não é texto UTF-8 legível — sem diff para mostrar.
      </p>
    );
  }

  const { hunks } = diff;

  if (hunks.length === 0) {
    return <p className="px-3 py-2 text-sm text-n-9">sem mudança de conteúdo neste arquivo.</p>;
  }

  return (
    <div>
      <ModeToggle mode={mode} onChange={setMode} />
      {mode === "unified" ? (
        <UnifiedView
          hunks={hunks}
          action={action}
          destructive={destructive}
          picks={picks}
          onToggleLine={toggleLine}
          onClearHunk={clearHunk}
        />
      ) : (
        <SideBySideView hunks={hunks} action={action} destructive={destructive} />
      )}
    </div>
  );
}

export function FileDiffView({ repoId, oid, path }: { repoId: string; oid: string; path: string }) {
  const diff = useFileDiff(repoId, oid, path);

  if (diff.isPending) {
    return <p className="px-3 py-2 text-sm text-n-8">carregando…</p>;
  }

  if (diff.isError) {
    return <p className="px-3 py-2 text-sm text-error">{(diff.error as Error).message}</p>;
  }

  return <DiffView diff={diff.data} />;
}
