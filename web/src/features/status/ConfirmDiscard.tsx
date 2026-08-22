/* A confirmação do descarte.
 *
 * O `BLOCO-E.md` é explícito: **descarte sempre confirma**, mesmo em hunk pequeno. É a única
 * operação do bloco que destrói trabalho sem reflog para socorrer — um commit errado se emenda,
 * um stage errado se desfaz, mas um arquivo sobrescrito pelo conteúdo do índice não existe em
 * lugar nenhum depois.
 *
 * Por isso a confirmação **nomeia o que será perdido**, um caminho por linha, em vez de
 * perguntar "tem certeza?". E o foco nasce no botão de cancelar: um Enter distraído não pode
 * ser o gesto que apaga.
 */

export function ConfirmDiscard({
  title,
  items,
  pending,
  onConfirm,
  onCancel,
}: {
  /** O que a ação é, em uma linha. */
  title: string;
  /** Exatamente o que será perdido — caminhos, ou a descrição dos trechos. */
  items: string[];
  pending: boolean;
  onConfirm: () => void;
  onCancel: () => void;
}) {
  return (
    <div
      role="alertdialog"
      aria-label={title}
      onKeyDown={(event) => {
        if (event.key !== "Escape") return;
        event.preventDefault();
        onCancel();
      }}
      className="shrink-0 border-t border-n-5 bg-n-2 px-3 py-2"
    >
      <p className="text-sm text-n-11">{title}</p>
      <ul className="mt-1 max-h-24 overflow-y-auto">
        {items.map((item) => (
          <li key={item} className="text-xs font-mono text-del truncate">
            {item}
          </li>
        ))}
      </ul>
      <p className="mt-1 text-xs text-n-8">
        isto não tem volta — o que for descartado não fica no reflog nem em lugar nenhum.
      </p>
      <div className="mt-1 flex items-center gap-2">
        {/* Cancelar primeiro e com o foco: um Enter distraído fecha em vez de apagar. */}
        <button
          autoFocus
          type="button"
          onClick={onCancel}
          className="px-2 py-0.5 border border-n-6 rounded-sm text-sm text-n-11 hover:bg-n-3 transition-colors duration-(--duration-fast)"
        >
          cancelar
        </button>
        <button
          type="button"
          onClick={onConfirm}
          disabled={pending}
          className="px-2 py-0.5 border border-n-6 rounded-sm text-sm text-del hover:bg-del-bg disabled:text-n-8 transition-colors duration-(--duration-fast)"
        >
          descartar
        </button>
      </div>
    </div>
  );
}
