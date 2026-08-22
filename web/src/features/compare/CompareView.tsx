/* Comparar dois pontos quaisquer do histórico — dois commits, duas branches, uma tag e uma
 * branch.
 *
 * Os dois lados são campos de texto **com sugestão** (as refs do repositório), não seletores
 * fechados: o valor aceito é qualquer revisão que o git entenda (`HEAD~2`, um hash colado,
 * `origin/main`), e um seletor de lista fecharia a porta para todas elas sem ganhar nada.
 *
 * A leitura é sempre "do primeiro para o segundo": o que apareceria como adição se o lado
 * esquerdo virasse o direito. É o mesmo sentido de `git diff a b`.
 */

import { useState } from "react";

import type { FileChange } from "../../lib/api-types";
import { useCompare, useCompareFileDiff, useCompareSelection } from "../../lib/compare";
import { useRefs } from "../../lib/repo";
import { DiffView } from "../log/FileDiffView";

function RevInput({
  label,
  value,
  onChange,
  options,
}: {
  label: string;
  value: string;
  onChange: (value: string) => void;
  options: string[];
}) {
  return (
    <label className="flex items-baseline gap-1 min-w-0">
      <span className="text-xs text-n-8 shrink-0">{label}</span>
      <input
        type="text"
        value={value}
        onChange={(event) => onChange(event.target.value)}
        list="porc-revisoes"
        spellCheck={false}
        placeholder="branch, tag ou hash"
        className="min-w-0 w-40 px-1 bg-n-0 border border-n-6 rounded-sm text-sm font-mono text-n-11 placeholder:text-n-8 outline-none focus-visible:border-n-8"
      />
      {/* Uma `datalist` só para os dois campos: a lista de refs é a mesma dos dois lados. */}
      <datalist id="porc-revisoes">
        {options.map((option) => (
          <option key={option} value={option} />
        ))}
      </datalist>
    </label>
  );
}

function FileRow({ file, onOpen }: { file: FileChange; onOpen: () => void }) {
  const label =
    file.oldPath && file.oldPath !== file.path ? `${file.oldPath} → ${file.path}` : file.path;

  return (
    <li>
      <button
        type="button"
        onClick={onOpen}
        className="flex items-baseline gap-2 w-full px-3 py-0.5 text-left hover:bg-n-3 transition-colors duration-(--duration-fast)"
      >
        <span className="text-sm font-mono text-n-10 truncate flex-1 min-w-0">{label}</span>
        {file.binary ? (
          <span className="text-xs text-n-8 shrink-0">binário</span>
        ) : (
          <span className="text-xs font-mono shrink-0">
            <span className="text-add">+{file.insertions}</span>{" "}
            <span className="text-del">-{file.deletions}</span>
          </span>
        )}
      </button>
    </li>
  );
}

export function CompareView({ repoId }: { repoId: string }) {
  const from = useCompareSelection((state) => state.from);
  const to = useCompareSelection((state) => state.to);
  const set = useCompareSelection((state) => state.set);
  const swap = useCompareSelection((state) => state.swap);

  const [openFile, setOpenFile] = useState<string | null>(null);
  const comparison = useCompare(repoId, from, to);
  const diff = useCompareFileDiff(repoId, from, to, openFile);
  const options = (useRefs(repoId).data ?? []).map((marker) => marker.name);

  // Ajuste durante o render, como no `CommitDetail`: trocar os lados fecha o arquivo aberto,
  // porque o diff dele só faz sentido dentro da comparação de onde foi aberto.
  const [openedFor, setOpenedFor] = useState(`${from}:${to}`);
  if (openedFor !== `${from}:${to}`) {
    setOpenedFor(`${from}:${to}`);
    setOpenFile(null);
  }

  return (
    <div className="flex-1 min-w-0 min-h-0 flex flex-col">
      <div className="flex items-center gap-2 px-3 h-8 border-b border-n-5 shrink-0">
        <RevInput label="de" value={from} onChange={(rev) => set("from", rev)} options={options} />
        <button
          type="button"
          onClick={swap}
          title="trocar os lados"
          className="text-xs px-1.5 py-0.5 rounded-sm border border-n-6 text-n-9 hover:text-n-11 shrink-0 transition-colors duration-(--duration-fast)"
        >
          ⇄
        </button>
        <RevInput label="para" value={to} onChange={(rev) => set("to", rev)} options={options} />
      </div>

      {comparison.isPending && <p className="px-3 py-2 text-sm text-n-8">comparando…</p>}
      {comparison.isError && (
        <p className="px-3 py-2 text-sm text-error">{(comparison.error as Error).message}</p>
      )}

      {comparison.isSuccess && (
        <>
          <div className="px-3 py-1 border-b border-n-5 shrink-0 text-xs font-mono text-n-9">
            <span className="text-n-8">{comparison.data.from.slice(0, 8)}</span> →{" "}
            <span className="text-n-8">{comparison.data.to.slice(0, 8)}</span> ·{" "}
            {comparison.data.files.length} arquivo
            {comparison.data.files.length === 1 ? "" : "s"} ·{" "}
            <span className="text-add">+{comparison.data.insertions}</span>{" "}
            <span className="text-del">-{comparison.data.deletions}</span>
          </div>

          {openFile !== null ? (
            <div className="flex-1 min-h-0 overflow-y-auto">
              <button
                type="button"
                onClick={() => setOpenFile(null)}
                className="flex items-center gap-2 w-full px-3 py-1.5 border-b border-n-5 text-left hover:bg-n-3 transition-colors duration-(--duration-fast)"
              >
                <span className="text-n-9">←</span>
                <span className="font-mono text-xs text-n-10 truncate">{openFile}</span>
              </button>
              {diff.isPending && <p className="px-3 py-2 text-sm text-n-8">carregando…</p>}
              {diff.isError && (
                <p className="px-3 py-2 text-sm text-error">{(diff.error as Error).message}</p>
              )}
              {/* Sem ação nenhuma nos hunks: comparar é leitura, e não há para onde mover um
                  trecho entre dois pontos do histórico. */}
              {diff.isSuccess && <DiffView diff={diff.data} />}
            </div>
          ) : comparison.data.files.length === 0 ? (
            <p className="px-3 py-2 text-sm text-n-8">os dois lados têm exatamente o mesmo conteúdo.</p>
          ) : (
            <ul className="flex-1 min-h-0 overflow-y-auto">
              {comparison.data.files.map((file) => (
                <FileRow key={file.path} file={file} onOpen={() => setOpenFile(file.path)} />
              ))}
            </ul>
          )}
        </>
      )}
    </div>
  );
}
