/* Clonar um repositório na pasta em que o navegador está.
 *
 * O formulário some assim que o job começa: daí em diante quem conta a história é o painel de
 * jobs, com progresso e botão de cancelar. Manter o formulário aberto sugeriria que ainda há algo
 * a preencher.
 *
 * O nome da pasta fica vazio por padrão e o servidor o deriva da URL. Preenchê-lo aqui, no
 * cliente, criaria duas regras de derivação para divergirem — e é o servidor que valida.
 */

import { useEffect, useRef, useState } from "react";

import { useCloneRepo } from "../../lib/jobs";

export interface CloneRepoProps {
  /** Pasta onde o clone cai — a que o navegador está mostrando. */
  parent: string | null;
  open: boolean;
  onClose: () => void;
}

const FIELD =
  "px-2 py-0.5 bg-n-0 border border-n-6 rounded-sm text-base text-n-11 placeholder:text-n-8 outline-none focus-visible:border-n-8";

export function CloneRepo({ parent, open, onClose }: CloneRepoProps) {
  const [url, setUrl] = useState("");
  const [folder, setFolder] = useState("");
  const [branch, setBranch] = useState("");
  const [depth, setDepth] = useState("");
  const [remote, setRemote] = useState("");
  const [submodules, setSubmodules] = useState(false);

  const clone = useCloneRepo();
  const urlRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) urlRef.current?.focus();
  }, [open]);

  if (!open || !parent) return null;

  function submit(event: React.FormEvent) {
    event.preventDefault();
    if (!parent || !url.trim()) return;

    const parsed = Number(depth);

    clone.mutate(
      {
        url: url.trim(),
        path: parent,
        folder: folder.trim() || undefined,
        branch: branch.trim() || undefined,
        depth: Number.isInteger(parsed) && parsed > 0 ? parsed : undefined,
        remote: remote.trim() || undefined,
        recurseSubmodules: submodules,
      },
      { onSuccess: onClose },
    );
  }

  return (
    <form
      onSubmit={submit}
      onKeyDown={(event) => event.key === "Escape" && onClose()}
      className="shrink-0 border-t border-n-5 bg-n-1 px-3 py-2"
    >
      <p className="text-xs text-n-8 truncate">clonar para dentro de {parent}</p>

      <div className="mt-1 flex items-center gap-2">
        <input
          ref={urlRef}
          value={url}
          onChange={(event) => setUrl(event.target.value)}
          placeholder="https://… ou git@…"
          className={`flex-1 min-w-0 font-mono ${FIELD}`}
        />
        <input
          value={folder}
          onChange={(event) => setFolder(event.target.value)}
          placeholder="pasta (da URL)"
          className={`w-44 ${FIELD}`}
        />
        <button
          type="submit"
          disabled={!url.trim() || clone.isPending}
          className="px-2 py-0.5 border border-n-6 rounded-sm text-sm text-n-11 hover:bg-n-3 disabled:text-n-8 disabled:hover:bg-transparent transition-colors duration-(--duration-fast)"
        >
          clonar
        </button>
        <button
          type="button"
          onClick={onClose}
          className="text-sm text-n-9 hover:text-n-11 transition-colors duration-(--duration-fast)"
        >
          cancelar
        </button>
      </div>

      <div className="mt-1 flex items-center gap-2 text-sm text-n-9">
        <input
          value={branch}
          onChange={(event) => setBranch(event.target.value)}
          placeholder="branch"
          className={`w-40 font-mono ${FIELD}`}
        />
        <input
          value={depth}
          onChange={(event) => setDepth(event.target.value)}
          placeholder="--depth"
          inputMode="numeric"
          className={`w-24 font-mono ${FIELD}`}
        />
        <input
          value={remote}
          onChange={(event) => setRemote(event.target.value)}
          placeholder="remote (origin)"
          className={`w-40 font-mono ${FIELD}`}
        />
        <label className="flex items-center gap-1 select-none">
          <input
            type="checkbox"
            checked={submodules}
            onChange={(event) => setSubmodules(event.target.checked)}
            className="accent-n-9"
          />
          submódulos
        </label>
      </div>

      {clone.isError && (
        <p className="mt-1 text-sm text-error">{(clone.error as Error).message}</p>
      )}
    </form>
  );
}
