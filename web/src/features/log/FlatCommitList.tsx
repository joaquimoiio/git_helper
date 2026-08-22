/* Uma lista achatada de commits, sem grafo: a forma que a busca (Passo 43) e o filtro por
 * caminho (Passo 45) compartilham — os dois produzem um conjunto de commits que não é
 * contíguo no histórico, então desenhar lanes para ele não faz sentido nenhum.
 *
 * `aria-label` fica com quem chama: "resultados da busca" e "commits que tocam o arquivo" são
 * anúncios diferentes para leitor de tela, mesmo com a mesma lista por baixo.
 */

import type { SearchHit } from "../../lib/api-types";
import { useCommitSelection } from "../../lib/commit";
import { relativeTime } from "../../lib/log";

export function FlatCommitList({ hits, ariaLabel }: { hits: SearchHit[]; ariaLabel: string }) {
  const oid = useCommitSelection((state) => state.oid);
  const select = useCommitSelection((state) => state.select);

  return (
    <div className="flex-1 min-h-0 overflow-y-auto" role="listbox" aria-label={ariaLabel}>
      {hits.map((hit) => (
        <div
          key={hit.oid}
          role="option"
          aria-selected={hit.oid === oid}
          onClick={() => select(hit.oid)}
          className={`flex items-baseline gap-3 px-3 py-0.5 cursor-default transition-colors duration-(--duration-fast) ${
            hit.oid === oid ? "bg-n-4" : "hover:bg-n-3"
          }`}
        >
          <span className="text-sm font-mono text-n-8 shrink-0">{hit.oid.slice(0, 7)}</span>
          <span className="text-sm text-n-11 truncate flex-1 min-w-0">{hit.summary}</span>
          <span className="text-sm text-n-9 truncate shrink-0 max-w-32">{hit.author}</span>
          <span className="text-xs text-n-8 shrink-0 w-20 text-right">{relativeTime(hit.time)}</span>
        </div>
      ))}
    </div>
  );
}
