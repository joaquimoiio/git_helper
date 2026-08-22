/* O diff de um arquivo: hunks estruturados, modo unificado ou lado a lado, realce leve
 * (fundo do token de adição/remoção — sem realce de sintaxe, que é biblioteca e peso que o
 * bloco não pediu).
 *
 * Binário e conteúdo não-UTF8 têm aviso próprio, nunca tentam desenhar um hunk que não existe.
 */

import { useState } from "react";

import type { DiffHunk, DiffLine } from "../../lib/api-types";
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

function HunkHeader({ header }: { header: string }) {
  return <div className="px-2 py-0.5 bg-n-2 text-n-8 whitespace-pre">{header}</div>;
}

function UnifiedLine({ line }: { line: DiffLine }) {
  const { bg, fg } = lineStyle(line.kind);
  const marker = line.kind === "addition" ? "+" : line.kind === "deletion" ? "-" : " ";

  return (
    <div className={`flex whitespace-pre ${bg}`}>
      <span className="w-9 shrink-0 text-right pr-1 text-n-8 select-none">{line.oldLineno ?? ""}</span>
      <span className="w-9 shrink-0 text-right pr-1 text-n-8 select-none">{line.newLineno ?? ""}</span>
      <span className={`pl-1 ${fg}`}>
        {marker}
        {line.content}
      </span>
    </div>
  );
}

function UnifiedView({ hunks }: { hunks: DiffHunk[] }) {
  return (
    <div className="overflow-x-auto font-mono text-xs">
      {hunks.map((hunk, hunkIndex) => (
        <div key={hunkIndex}>
          <HunkHeader header={hunk.header} />
          {hunk.lines.map((line, lineIndex) => (
            // A posição na lista é a chave estável: hunks não reordenam linhas entre eles.
            <UnifiedLine key={lineIndex} line={line} />
          ))}
        </div>
      ))}
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

function SideBySideView({ hunks }: { hunks: DiffHunk[] }) {
  return (
    <div className="overflow-x-auto font-mono text-xs">
      {hunks.map((hunk, hunkIndex) => (
        <div key={hunkIndex}>
          <HunkHeader header={hunk.header} />
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

export function FileDiffView({ repoId, oid, path }: { repoId: string; oid: string; path: string }) {
  const diff = useFileDiff(repoId, oid, path);
  const [mode, setMode] = useState<"unified" | "side-by-side">("unified");

  if (diff.isPending) {
    return <p className="px-3 py-2 text-sm text-n-8">carregando…</p>;
  }

  if (diff.isError) {
    return <p className="px-3 py-2 text-sm text-error">{(diff.error as Error).message}</p>;
  }

  if (diff.data.kind === "binary") {
    return <p className="px-3 py-2 text-sm text-n-9">arquivo binário — sem diff de texto.</p>;
  }

  if (diff.data.kind === "notUtf8") {
    return (
      <p className="px-3 py-2 text-sm text-n-9">
        este arquivo não é texto UTF-8 legível — sem diff para mostrar.
      </p>
    );
  }

  const { hunks } = diff.data;

  if (hunks.length === 0) {
    return <p className="px-3 py-2 text-sm text-n-9">sem mudança de conteúdo neste arquivo.</p>;
  }

  return (
    <div>
      <ModeToggle mode={mode} onChange={setMode} />
      {mode === "unified" ? <UnifiedView hunks={hunks} /> : <SideBySideView hunks={hunks} />}
    </div>
  );
}
