/* Divisória arrastável entre dois painéis.
 *
 * Visualmente é a borda de 1px e nada mais; a área de agarrar é maior que o traço, senão
 * acertar 1px com o mouse vira exercício de pontaria. Por isso o elemento tem largura de
 * 5px e é puxado meio pixel para dentro com margem negativa — o layout continua contando
 * só 1px de separação.
 *
 * `setPointerCapture` e não listeners no `window`: o arrasto continua funcionando quando o
 * ponteiro sai do elemento, e acaba sozinho se o botão for solto fora da janela.
 */

import { useRef, type PointerEvent as ReactPointerEvent } from "react";

import { useLayout, type Panel } from "../lib/layout";

interface SplitterProps {
  panel: Panel;
  /** De que lado do painel a divisória está — decide o sinal do arrasto. */
  edge: "start" | "end";
}

export function Splitter({ panel, edge }: SplitterProps) {
  const setWidth = useLayout((state) => state.setWidth);
  const toggle = useLayout((state) => state.toggle);
  const origin = useRef({ x: 0, width: 0 });

  function onPointerDown(event: ReactPointerEvent<HTMLDivElement>) {
    // Só botão principal: arrastar com o botão do meio ou direito não é gesto de redimensionar.
    if (event.button !== 0) return;

    const { sidebarWidth, detailWidth } = useLayout.getState();
    origin.current = { x: event.clientX, width: panel === "sidebar" ? sidebarWidth : detailWidth };

    event.currentTarget.setPointerCapture(event.pointerId);
    event.preventDefault();
  }

  function onPointerMove(event: ReactPointerEvent<HTMLDivElement>) {
    if (!event.currentTarget.hasPointerCapture(event.pointerId)) return;

    const delta = event.clientX - origin.current.x;
    // Painel à direita cresce quando o ponteiro vai para a esquerda.
    setWidth(panel, origin.current.width + (edge === "end" ? delta : -delta));
  }

  return (
    <div
      role="separator"
      aria-orientation="vertical"
      aria-label={`redimensionar painel ${panel}`}
      tabIndex={-1}
      onPointerDown={onPointerDown}
      onPointerMove={onPointerMove}
      onDoubleClick={() => toggle(panel)}
      className="relative z-10 w-[5px] -mx-[2px] shrink-0 cursor-col-resize select-none touch-none
                 bg-transparent hover:bg-n-7 transition-colors duration-(--duration-fast)"
    />
  );
}
