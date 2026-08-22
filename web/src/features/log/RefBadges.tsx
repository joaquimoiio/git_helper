/* Marcadores de ponta numa linha do log: branch, remota, tag e `HEAD` destacado.
 *
 * Sem arco-íris — a diferença entre os tipos é traço (borda sólida contra tracejada) e peso
 * (a ponta do `HEAD` fica preenchida), nunca cor. `shrink-0` com `max-w` e `overflow-hidden`:
 * um commit com muitas tags trunca a lista em vez de espremer o resto da linha.
 */

import type { RefMarker } from "../../lib/api-types";

export function RefBadges({ markers }: { markers: RefMarker[] | undefined }) {
  if (!markers || markers.length === 0) return null;

  return (
    <span className="flex items-center gap-1 shrink-0 max-w-48 overflow-hidden">
      {markers.map((marker) => (
        <span
          key={`${marker.kind}-${marker.name}`}
          className={`text-xs font-mono px-1 rounded-sm border truncate ${
            marker.kind === "tag" ? "border-dashed" : "border-solid"
          } ${
            marker.isHead
              ? "border-n-7 bg-n-6 text-n-11 font-medium"
              : marker.kind === "remote"
                ? "border-n-6 text-n-9"
                : "border-n-6 text-n-10"
          }`}
        >
          {marker.name}
        </span>
      ))}
    </span>
  );
}
