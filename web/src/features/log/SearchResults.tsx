/* Resultados da busca por mensagem/autor: uma lista achatada, sem grafo.
 *
 * A busca cobre o histórico indexado inteiro, não só o que o log rolou até agora — não faz
 * sentido desenhar lanes para um subconjunto que nem é contíguo no grafo original. É a mesma
 * ideia de qualquer busca de commit em ferramenta gráfica: resultado é lista, grafo é o log
 * sem filtro.
 */

import { useEffect } from "react";

import { useCommitSelection } from "../../lib/commit";
import { useSearch } from "../../lib/search";
import { FlatCommitList } from "./FlatCommitList";

/* Mesma forma que `is_hash_like` no `porc-index`: um único token hex de 4-40 caracteres é o
 * formato de um hash de commit, curto ou completo. */
const HASH_LIKE = /^[0-9a-f]{4,40}$/i;

export function SearchResults({ repoId, query }: { repoId: string; query: string }) {
  const results = useSearch(repoId, query);
  const select = useCommitSelection((state) => state.select);

  // "Colar um hash curto salta direto para o commit": se a busca inteira parece um hash e
  // achou exatamente um, esse é o commit que o usuário queria — preenche o painel de detalhe
  // sem esperar um clique. Query ambígua ou com mais de um resultado não arrisca adivinhar.
  const hashHit = HASH_LIKE.test(query.trim()) && results.data?.length === 1 ? results.data[0] : undefined;
  useEffect(() => {
    if (hashHit) select(hashHit.oid);
  }, [hashHit, select]);

  if (results.isPending) {
    return <p className="px-3 py-2 text-sm text-n-8">buscando…</p>;
  }

  if (results.isError) {
    return <p className="px-3 py-2 text-sm text-error">{(results.error as Error).message}</p>;
  }

  if (results.data.length === 0) {
    return <p className="px-3 py-2 text-sm text-n-9">nenhum commit encontrado.</p>;
  }

  return <FlatCommitList hits={results.data} ariaLabel="resultados da busca" />;
}
