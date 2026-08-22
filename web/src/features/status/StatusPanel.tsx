/* O painel de status: os três grupos do `git status`, com stage e unstage por linha e em
 * lote, tudo alcançável pelo teclado.
 *
 * Uma lista só, achatada, com cabeçalho de grupo no meio — não três listas independentes: o
 * cursor atravessa os grupos com uma tecla só, que é o que faz "marcar cinco arquivos
 * espalhados e stagear" ser um gesto e não três.
 *
 * `marked` é a seleção múltipla (barra de espaço); o cursor sozinho vale como seleção de um
 * quando nada está marcado — assim `s` funciona sem cerimônia no caso comum de um arquivo só.
 */

import { useEffect, useMemo, useRef, useState } from "react";

import type { StatusEntry, StatusKind, WorktreeStatus } from "../../lib/api-types";
import type { StatusGroup } from "../../lib/status";
import { useDiscard, useStage, useStatus, useStatusSelection, useUnstage } from "../../lib/status";
import { CommitBox } from "./CommitBox";
import { ConfirmDiscard } from "./ConfirmDiscard";

/** Uma letra por estado, como o `git status` curto — a coluna fica com largura fixa. */
const KIND_LETTER: Record<StatusKind, string> = {
  added: "A",
  modified: "M",
  deleted: "D",
  renamed: "R",
  copied: "C",
  typechange: "T",
  unmerged: "U",
  untracked: "?",
};

const GROUP_LABEL: Record<StatusGroup, string> = {
  staged: "preparado para o commit",
  unstaged: "modificado, fora do commit",
  untracked: "não rastreado",
};

const STATE_LABEL: Record<Exclude<WorktreeStatus["state"], "clean">, string> = {
  merge: "merge em andamento",
  rebase: "rebase em andamento",
  cherryPick: "cherry-pick em andamento",
  revert: "revert em andamento",
  bisect: "bisect em andamento",
};

interface Row {
  group: StatusGroup;
  entry: StatusEntry;
}

/** `group:path` — o par é a identidade da linha: o mesmo caminho pode estar em dois grupos. */
function idOf(row: Row) {
  return `${row.group}:${row.entry.path}`;
}

function flatten(status: WorktreeStatus): Row[] {
  return [
    ...status.staged.map((entry) => ({ group: "staged" as const, entry })),
    ...status.unstaged.map((entry) => ({ group: "unstaged" as const, entry })),
    ...status.untracked.map((entry) => ({ group: "untracked" as const, entry })),
  ];
}

function BranchLine({ status }: { status: WorktreeStatus }) {
  const { branch } = status;

  return (
    <div className="flex items-baseline gap-2 px-3 h-8 border-b border-n-5 shrink-0">
      <span className="text-sm font-mono text-n-11 truncate">
        {branch.head ?? branch.oid?.slice(0, 7) ?? "sem commits"}
      </span>
      {branch.detached && <span className="text-xs text-n-8">detached</span>}
      {branch.upstream && (
        <span className="text-xs font-mono text-n-9 truncate">
          {branch.upstream}
          {branch.ahead > 0 && ` ↑${branch.ahead}`}
          {branch.behind > 0 && ` ↓${branch.behind}`}
        </span>
      )}
      {status.state !== "clean" && (
        <span className="text-xs text-conflict">{STATE_LABEL[status.state]}</span>
      )}
      <span className="flex-1" />
      <span className="text-xs font-mono text-n-8">
        espaço marca · s prepara · u desfaz · a grupo · d descarta · c mensagem
      </span>
    </div>
  );
}

export function StatusPanel({ repoId }: { repoId: string }) {
  const { data, isPending, isError, error } = useStatus(repoId);
  const stage = useStage(repoId);
  const unstage = useUnstage(repoId);
  const discard = useDiscard(repoId);

  const select = useStatusSelection((state) => state.select);
  const selectedPath = useStatusSelection((state) => state.path);
  const selectedGroup = useStatusSelection((state) => state.group);

  const [cursor, setCursor] = useState(0);
  const [marked, setMarked] = useState<ReadonlySet<string>>(new Set());
  // Caminhos esperando confirmação de descarte. `null` é "não há nada perguntado".
  const [confirming, setConfirming] = useState<string[] | null>(null);
  const listRef = useRef<HTMLDivElement>(null);
  const messageRef = useRef<HTMLTextAreaElement>(null);

  const rows = useMemo(() => (data ? flatten(data) : []), [data]);

  // O teclado é o caminho principal deste painel: sem foco, `s` e `u` não chegariam a lugar
  // nenhum e o usuário teria que clicar antes de poder usar o atalho.
  useEffect(() => {
    listRef.current?.focus();
  }, []);

  // Ajuste durante o render, não em efeito: a lista encolhe a cada stage, e um cursor fora do
  // fim desenharia uma linha destacada que não existe até o efeito rodar.
  const at = rows.length === 0 ? 0 : Math.min(cursor, rows.length - 1);
  if (at !== cursor) setCursor(at);

  // Uma seleção que sumiu do status (o arquivo foi stageado inteiro) não pode continuar
  // apontada: quem lê isto é o painel de detalhe, e o alvo não existe mais.
  const current = rows[at];
  const selectionAlive =
    selectedPath === null ||
    rows.some((row) => row.entry.path === selectedPath && row.group === selectedGroup);
  useEffect(() => {
    if (!selectionAlive && current) select(current.entry.path, current.group);
  }, [selectionAlive, current, select]);

  function pathsOf(rows: Row[]) {
    return [...new Set(rows.map((row) => row.entry.path))];
  }

  /** Marcados, ou o do cursor quando nada está marcado. */
  function target(): Row[] {
    const chosen = rows.filter((row) => marked.has(idOf(row)));
    if (chosen.length > 0) return chosen;
    return current ? [current] : [];
  }

  function apply(action: "stage" | "unstage") {
    const rows = target();
    if (rows.length === 0) return;

    (action === "stage" ? stage : unstage).mutate(pathsOf(rows));
    setMarked(new Set());
  }

  function move(index: number) {
    const clamped = Math.min(Math.max(index, 0), Math.max(rows.length - 1, 0));
    setCursor(clamped);

    const row = rows[clamped];
    if (row) select(row.entry.path, row.group);
  }

  function toggle(row: Row) {
    setMarked((previous) => {
      const next = new Set(previous);
      const id = idOf(row);
      if (!next.delete(id)) next.add(id);
      return next;
    });
  }

  function markGroup(group: StatusGroup) {
    const ids = rows.filter((row) => row.group === group).map(idOf);
    setMarked((previous) => {
      // Já todos marcados: a mesma tecla desmarca. Um `a` que só liga obrigaria a Esc.
      const all = ids.every((id) => previous.has(id));
      const next = new Set(previous);
      for (const id of ids) {
        if (all) next.delete(id);
        else next.add(id);
      }
      return next;
    });
  }

  function onKeyDown(event: React.KeyboardEvent) {
    if (event.metaKey || event.ctrlKey || event.altKey) return;
    if (rows.length === 0) return;

    switch (event.key) {
      case "ArrowDown":
      case "j":
        event.preventDefault();
        return move(at + 1);
      case "ArrowUp":
      case "k":
        event.preventDefault();
        return move(at - 1);
      case "Home":
        event.preventDefault();
        return move(0);
      case "End":
        event.preventDefault();
        return move(rows.length - 1);
      case " ":
        event.preventDefault();
        if (current) toggle(current);
        return move(at + 1);
      case "s":
        event.preventDefault();
        return apply("stage");
      case "u":
        event.preventDefault();
        return apply("unstage");
      case "a":
        event.preventDefault();
        if (current) markGroup(current.group);
        return;
      case "d":
        // Não descarta: **abre a confirmação**. A tecla é o começo do gesto, nunca o fim dele.
        event.preventDefault();
        if (target().length > 0) setConfirming(pathsOf(target()));
        return;
      case "c":
        // Do cursor direto para a mensagem, sem tirar a mão do teclado — é o gesto seguinte
        // depois de preparar o que vai entrar.
        event.preventDefault();
        messageRef.current?.focus();
        return;
      case "Escape":
        event.preventDefault();
        setMarked(new Set());
        return;
    }
  }

  if (isPending) {
    return <p className="px-3 py-2 text-sm text-n-8">carregando…</p>;
  }

  if (isError) {
    return <p className="px-3 py-2 text-sm text-error">{(error as Error).message}</p>;
  }

  const failed = stage.error ?? unstage.error ?? discard.error;

  return (
    <div className="flex-1 min-w-0 min-h-0 flex flex-col">
      <BranchLine status={data} />
      {failed && <p className="px-3 py-1 text-sm text-error bg-error-bg shrink-0">{failed.message}</p>}
      <div
        ref={listRef}
        tabIndex={0}
        role="listbox"
        aria-label="arquivos modificados"
        aria-activedescendant={current ? `status-${at}` : undefined}
        onKeyDown={onKeyDown}
        className="flex-1 min-h-0 overflow-y-auto outline-none"
      >
        {rows.length === 0 && (
          <p className="px-3 py-2 text-sm text-n-8">nada para commitar — a worktree está limpa.</p>
        )}
        {(["staged", "unstaged", "untracked"] as StatusGroup[]).map((group) => {
          const inGroup = rows.filter((row) => row.group === group);
          if (inGroup.length === 0) return null;

          return (
            <section key={group}>
              <h2 className="flex items-baseline gap-2 px-3 pt-2 pb-0.5 text-xs uppercase tracking-[0.1em] text-n-8">
                {GROUP_LABEL[group]}
                <span className="font-mono normal-case tracking-normal">{inGroup.length}</span>
              </h2>
              {inGroup.map((row) => {
                const index = rows.indexOf(row);
                const isCursor = index === at;
                const isMarked = marked.has(idOf(row));

                return (
                  <div
                    key={idOf(row)}
                    id={`status-${index}`}
                    role="option"
                    aria-selected={isCursor}
                    onClick={() => move(index)}
                    onDoubleClick={() => apply(group === "staged" ? "unstage" : "stage")}
                    className={`flex items-baseline gap-2 px-3 py-0.5 cursor-default transition-colors duration-(--duration-fast) ${
                      isCursor ? "bg-n-4" : "hover:bg-n-3"
                    }`}
                  >
                    <span className="w-3 shrink-0 text-sm font-mono text-n-9 select-none">
                      {isMarked ? "•" : ""}
                    </span>
                    <span
                      className={`w-3 shrink-0 text-sm font-mono ${
                        group === "staged" ? "text-add" : "text-del"
                      }`}
                    >
                      {KIND_LETTER[row.entry.kind]}
                    </span>
                    <span className="text-sm font-mono text-n-11 truncate flex-1 min-w-0">
                      {row.entry.path}
                    </span>
                    {row.entry.oldPath && (
                      <span className="text-xs font-mono text-n-8 truncate shrink-0 max-w-64">
                        ← {row.entry.oldPath}
                      </span>
                    )}
                  </div>
                );
              })}
            </section>
          );
        })}
      </div>
      {confirming && (
        <ConfirmDiscard
          title={`descartar as mudanças de ${confirming.length} arquivo${confirming.length > 1 ? "s" : ""}?`}
          items={confirming}
          pending={discard.isPending}
          onCancel={() => setConfirming(null)}
          onConfirm={() => {
            discard.mutate(confirming);
            setConfirming(null);
            setMarked(new Set());
            // O foco volta para a lista: quem confirmou pelo teclado continua no teclado.
            listRef.current?.focus();
          }}
        />
      )}
      <CommitBox
        repoId={repoId}
        canCommit={data.staged.length > 0}
        headOid={data.branch.oid}
        textareaRef={messageRef}
      />
    </div>
  );
}
