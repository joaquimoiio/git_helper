/* Criar um repositório na pasta em que o navegador está.
 *
 * Fica fechado até alguém pedir (`n` no navegador, ou o botão): a tela inicial é para abrir o
 * que já existe, e um formulário sempre aberto competiria com a lista pela atenção.
 *
 * O campo de branch começa vazio de propósito. Vazio significa "usa o `init.defaultBranch` do
 * usuário" — preencher com "main" seria escolher por quem já escolheu.
 */

import { useEffect, useRef, useState } from "react";

import { useInitRepo } from "../../lib/repo";

export interface NewRepoProps {
  /** Pasta onde o repositório nasce — a que o navegador está mostrando. */
  parent: string | null;
  open: boolean;
  onClose: () => void;
}

export function NewRepo({ parent, open, onClose }: NewRepoProps) {
  const [name, setName] = useState("");
  const [branch, setBranch] = useState("");
  const init = useInitRepo();
  const nameRef = useRef<HTMLInputElement>(null);

  useEffect(() => {
    if (open) nameRef.current?.focus();
  }, [open]);

  if (!open || !parent) return null;

  function submit(event: React.FormEvent) {
    event.preventDefault();
    if (!parent || !name.trim()) return;

    init.mutate({
      path: parent,
      name: name.trim(),
      branch: branch.trim() || undefined,
    });
  }

  return (
    <form
      onSubmit={submit}
      onKeyDown={(event) => event.key === "Escape" && onClose()}
      className="shrink-0 border-t border-n-5 bg-n-1 px-3 py-2"
    >
      <p className="text-xs text-n-8 truncate">criar repositório em {parent}</p>

      <div className="mt-1 flex items-center gap-2">
        <input
          ref={nameRef}
          value={name}
          onChange={(event) => setName(event.target.value)}
          placeholder="nome da pasta"
          className="flex-1 min-w-0 px-2 py-0.5 bg-n-0 border border-n-6 rounded-sm text-base text-n-11 placeholder:text-n-8 outline-none focus-visible:border-n-8"
        />
        <input
          value={branch}
          onChange={(event) => setBranch(event.target.value)}
          placeholder="branch inicial (opcional)"
          className="w-56 px-2 py-0.5 bg-n-0 border border-n-6 rounded-sm text-base font-mono text-n-11 placeholder:text-n-8 outline-none focus-visible:border-n-8"
        />
        <button
          type="submit"
          disabled={!name.trim() || init.isPending}
          className="px-2 py-0.5 border border-n-6 rounded-sm text-sm text-n-11 hover:bg-n-3 disabled:text-n-8 disabled:hover:bg-transparent transition-colors duration-(--duration-fast)"
        >
          {init.isPending ? "criando…" : "criar"}
        </button>
        <button
          type="button"
          onClick={onClose}
          className="text-sm text-n-9 hover:text-n-11 transition-colors duration-(--duration-fast)"
        >
          cancelar
        </button>
      </div>

      {init.isError && (
        <p className="mt-1 text-sm text-error">{(init.error as Error).message}</p>
      )}
    </form>
  );
}
