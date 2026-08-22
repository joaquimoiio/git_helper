/* O grafo: um `<canvas>` só, sobreposto à lista virtualizada.
 *
 * Redesenha na mesma renderização que reposiciona as linhas de texto — o `useVirtualizer` já
 * dispara um re-render a cada passo de rolagem, e é esse mesmo re-render que entrega `items` e
 * `scrollOffset` atualizados. Não existe um segundo laço de animação: dois relógios
 * desenhando a mesma coisa é como o grafo fica defasado da lista.
 *
 * Lanes e arestas vêm prontas do servidor (Passo 37); aqui é só geometria e traço. Sem
 * arco-íris — a única cor é a do texto mudo do tema, e a única diferenciação é posição e peso.
 */

import { useEffect, useMemo, useRef } from "react";
import type { VirtualItem } from "@tanstack/react-virtual";

import type { Commit } from "../../lib/api-types";

const NODE_RADIUS = 2.5;
const SELECTED_NODE_RADIUS = 3.5;

/* Quando o pai de uma aresta ainda não chegou (página seguinte não carregada), a linha não
 * finge saber a distância até ele: desce um coto curto e marca o fim com um ponto. Se o
 * usuário rolar até lá, a aresta materializa sozinha — o mesmo desenho recalculado já encontra
 * o destino em `rowOf`, sem código extra para "completar" a aresta depois. */
const STUB_LENGTH = 22;

export interface LogGraphProps {
  commits: Commit[];
  items: VirtualItem[];
  scrollOffset: number;
  rowHeight: number;
  laneWidth: number;
  gutterWidth: number;
  selectedIndex: number;
}

export function LogGraph({
  commits,
  items,
  scrollOffset,
  rowHeight,
  laneWidth,
  gutterWidth,
  selectedIndex,
}: LogGraphProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null);
  const lineColorRef = useRef<HTMLSpanElement>(null);
  const nodeColorRef = useRef<HTMLSpanElement>(null);

  // oid → linha. Índice do vetor inteiro carregado, não só do visível — recalculado apenas
  // quando `commits` muda (nova página chegou), nunca por rolagem.
  const rowOf = useMemo(() => {
    const map = new Map<string, number>();
    commits.forEach((commit, index) => map.set(commit.oid, index));
    return map;
  }, [commits]);

  useEffect(() => {
    const canvas = canvasRef.current;
    if (!canvas) return;

    const height = canvas.clientHeight;
    const dpr = window.devicePixelRatio || 1;
    // A resolução do canvas segue o pixel físico da tela; o `scale` abaixo devolve as
    // coordenadas de desenho para pixels CSS, senão tudo sairia do tamanho em telas retina.
    canvas.width = Math.round(gutterWidth * dpr);
    canvas.height = Math.round(height * dpr);

    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    ctx.setTransform(dpr, 0, 0, dpr, 0, 0);
    ctx.clearRect(0, 0, gutterWidth, height);

    // Cor tirada do token do tema (via um elemento real, para `light-dark()` resolver certo),
    // não hardcoded — CSS canvas não entende `var()` nem `light-dark()` diretamente.
    const lineColor = lineColorRef.current ? getComputedStyle(lineColorRef.current).color : "#888";
    const nodeColor = nodeColorRef.current ? getComputedStyle(nodeColorRef.current).color : "#888";

    const x = (lane: number) => laneWidth / 2 + lane * laneWidth;
    const y = (row: number) => row * rowHeight - scrollOffset + rowHeight / 2;

    ctx.strokeStyle = lineColor;
    ctx.fillStyle = lineColor;

    for (const item of items) {
      const commit = commits[item.index];
      if (!commit) continue;

      const sourceX = x(commit.lane);
      const sourceY = y(item.index);

      commit.parentLanes.forEach((parentLane, k) => {
        // `null` é a fronteira de um clone raso: o pai não existe localmente, a linha não
        // continua para lugar nenhum — diferente do caso abaixo, em que a coluna é conhecida
        // mas o commit em si ainda não carregou.
        if (parentLane === null) return;

        const targetRow = rowOf.get(commit.parents[k]);
        ctx.lineWidth = item.index === selectedIndex ? 1.5 : 1;

        ctx.beginPath();
        ctx.moveTo(sourceX, sourceY);

        if (targetRow !== undefined) {
          ctx.lineTo(x(parentLane), y(targetRow));
          ctx.stroke();
          return;
        }

        const stubY = sourceY + STUB_LENGTH;
        ctx.lineTo(x(parentLane), stubY);
        ctx.stroke();

        ctx.beginPath();
        ctx.arc(x(parentLane), stubY, 1.5, 0, Math.PI * 2);
        ctx.fill();
      });
    }

    ctx.fillStyle = nodeColor;
    for (const item of items) {
      const commit = commits[item.index];
      if (!commit) continue;

      const radius = item.index === selectedIndex ? SELECTED_NODE_RADIUS : NODE_RADIUS;
      ctx.beginPath();
      ctx.arc(x(commit.lane), y(item.index), radius, 0, Math.PI * 2);
      ctx.fill();
    }
  }, [commits, items, scrollOffset, rowHeight, laneWidth, gutterWidth, selectedIndex, rowOf]);

  return (
    <>
      {/* Tamanho zero, fora do fluxo: existem só para `getComputedStyle` devolver a cor real
          do token neste tema, nunca são vistos. */}
      <span ref={lineColorRef} aria-hidden className="absolute h-0 w-0 overflow-hidden text-n-7" />
      <span ref={nodeColorRef} aria-hidden className="absolute h-0 w-0 overflow-hidden text-n-10" />
      <canvas
        ref={canvasRef}
        aria-hidden
        className="pointer-events-none absolute inset-y-0 left-0"
        style={{ width: gutterWidth, height: "100%" }}
      />
    </>
  );
}
