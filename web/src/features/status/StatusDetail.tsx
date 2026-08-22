/* O painel direito enquanto o centro mostra o status: o diff do arquivo sob o cursor, do lado
 * certo — o que está fora do commit ou o que já está dentro dele.
 *
 * Quem decide o lado é o grupo em que a linha está, não um seletor: a mesma linha de `MM`
 * aparece nos dois grupos justamente porque são dois diffs diferentes, e obrigar a escolher de
 * novo depois de já ter escolhido a linha seria perguntar duas vezes a mesma coisa.
 */

import { useState } from "react";

import type { HunkPick } from "../../lib/api-types";
import { DiffView } from "../log/FileDiffView";
import {
  sideOf,
  useApplyHunks,
  useDiscardHunks,
  useStatusSelection,
  useWorktreeDiff,
} from "../../lib/status";
import { ConfirmDiscard } from "./ConfirmDiscard";

const SIDE_LABEL = {
  unstaged: "fora do commit",
  staged: "preparado",
} as const;

/** O que acontece com um hunk clicado, de cada lado. O verbo é o da direção, não do lado. */
const ACTION_LABEL = {
  unstaged: "preparar",
  staged: "desfazer",
} as const;

export function StatusDetail({ repoId }: { repoId: string }) {
  const path = useStatusSelection((state) => state.path);
  const group = useStatusSelection((state) => state.group);

  const side = sideOf(group ?? "unstaged");
  const diff = useWorktreeDiff(repoId, side, path);
  const apply = useApplyHunks(repoId);
  const discard = useDiscardHunks(repoId);
  // Trechos esperando confirmação. Descarte **sempre** confirma, mesmo de um hunk pequeno.
  const [confirming, setConfirming] = useState<HunkPick[] | null>(null);

  if (path === null) {
    return <p className="px-3 py-2 text-sm text-n-9">selecione um arquivo no status.</p>;
  }

  return (
    <div>
      <div className="px-3 py-2 border-b border-n-5">
        <p className="text-sm font-mono text-n-11 break-all">{path}</p>
        <p className="mt-0.5 text-xs text-n-8">{SIDE_LABEL[side]}</p>
      </div>
      {apply.isError && (
        <p className="px-3 py-1 text-sm text-error bg-error-bg">{apply.error.message}</p>
      )}
      {discard.isError && (
        <p className="px-3 py-1 text-sm text-error bg-error-bg">{discard.error.message}</p>
      )}
      {confirming && (
        <ConfirmDiscard
          title="descartar este trecho do arquivo em disco?"
          items={confirming.map((pick) =>
            pick.lines
              ? `trecho ${pick.hunk + 1}, ${pick.lines.length} linha${pick.lines.length > 1 ? "s" : ""} — ${path}`
              : `trecho ${pick.hunk + 1} inteiro — ${path}`,
          )}
          pending={discard.isPending}
          onCancel={() => setConfirming(null)}
          onConfirm={() => {
            discard.mutate({ path, hunks: confirming });
            setConfirming(null);
          }}
        />
      )}
      {diff.isPending && <p className="px-3 py-2 text-sm text-n-8">carregando…</p>}
      {diff.isError && (
        <p className="px-3 py-2 text-sm text-error">{(diff.error as Error).message}</p>
      )}
      {diff.isSuccess && (
        <DiffView
          // Remonta ao trocar de arquivo ou de lado: sem isto, linhas escolhidas num arquivo
          // sobreviveriam para o seguinte e virariam um patch em cima do arquivo errado.
          key={`${path}:${side}`}
          diff={diff.data}
          action={{
            label: ACTION_LABEL[side],
            pending: apply.isPending,
            // Sem confirmação: nada aqui destrói trabalho — o arquivo em disco não é tocado
            // (`--cached`), e a volta é o mesmo botão do outro lado. Quem precisa de
            // confirmação é o discard (Passo 55).
            onApply: (hunks) => apply.mutate({ path, side, hunks }),
          }}
          // Só do lado de fora do commit: o que está no índice se desfaz com "desfazer", que
          // não perde nada. Oferecer descarte ali seria oferecer perda onde há uma saída sem
          // perda nenhuma.
          destructive={
            side === "unstaged"
              ? {
                  label: "descartar",
                  pending: discard.isPending,
                  onApply: (hunks) => setConfirming(hunks),
                }
              : undefined
          }
        />
      )}
    </div>
  );
}
