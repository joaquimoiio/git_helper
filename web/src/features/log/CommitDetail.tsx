/* Painel de detalhe: o que preenche a coluna direita ao selecionar uma linha do log.
 *
 * Mensagem completa, as duas assinaturas, pais clicáveis (navegam o histórico ali mesmo, sem
 * mexer no cursor do log — ver `lib/commit.ts`) e a lista de arquivos com diffstat. Clicar num
 * arquivo troca a lista pelo diff dele (`FileDiffView`) — o painel é estreito demais para
 * mostrar lista e patch ao mesmo tempo.
 */

import { useState } from "react";

import type { FileChange, Signature } from "../../lib/api-types";
import { formatSignatureDate, useCommitDetail, useCommitSelection } from "../../lib/commit";
import { FileDiffView } from "./FileDiffView";

function SignatureRow({ label, signature }: { label: string; signature: Signature }) {
  return (
    <div>
      <dt className="text-xs uppercase tracking-[0.1em] text-n-8">{label}</dt>
      <dd className="text-n-10 truncate">
        {signature.name} <span className="text-n-8">&lt;{signature.email}&gt;</span>
      </dd>
      <dd className="font-mono text-xs text-n-8">
        {formatSignatureDate(signature.time, signature.offset)}
      </dd>
    </div>
  );
}

function FileRow({ file, onOpen }: { file: FileChange; onOpen: () => void }) {
  const label = file.oldPath && file.oldPath !== file.path ? `${file.oldPath} → ${file.path}` : file.path;

  return (
    <li>
      <button
        type="button"
        onClick={onOpen}
        className="flex items-baseline gap-2 w-full px-3 py-0.5 text-left hover:bg-n-3 transition-colors duration-(--duration-fast)"
      >
        <span className="text-xs font-mono text-n-10 truncate flex-1 min-w-0">{label}</span>
        {file.binary ? (
          <span className="text-xs text-n-8 shrink-0">binário</span>
        ) : (
          <span className="text-xs font-mono shrink-0">
            <span className="text-add">+{file.insertions}</span>{" "}
            <span className="text-del">-{file.deletions}</span>
          </span>
        )}
      </button>
    </li>
  );
}

export function CommitDetail({ repoId }: { repoId: string }) {
  const oid = useCommitSelection((state) => state.oid);
  const select = useCommitSelection((state) => state.select);
  const detail = useCommitDetail(repoId, oid);
  const [openFile, setOpenFile] = useState<string | null>(null);
  // Ajuste durante a renderização, não `useEffect`: trocar de commit (linha do log ou clique
  // num pai) fecha o arquivo aberto, porque o diff só faz sentido no contexto de onde foi
  // aberto. Comparar contra o `oid` visto da última vez evita o passo extra de render que um
  // efeito síncrono causaria.
  const [oidOfOpenFile, setOidOfOpenFile] = useState(oid);
  if (oid !== oidOfOpenFile) {
    setOidOfOpenFile(oid);
    setOpenFile(null);
  }

  if (oid === null) {
    return <p className="px-3 py-2 text-sm text-n-9">selecione um commit no log.</p>;
  }

  if (detail.isPending) {
    return <p className="px-3 py-2 text-sm text-n-8">carregando…</p>;
  }

  if (detail.isError) {
    return <p className="px-3 py-2 text-sm text-error">{(detail.error as Error).message}</p>;
  }

  const commit = detail.data;

  if (openFile !== null) {
    return (
      <div className="text-sm">
        <button
          type="button"
          onClick={() => setOpenFile(null)}
          className="flex items-center gap-2 w-full px-3 py-1.5 border-b border-n-5 text-left hover:bg-n-3 transition-colors duration-(--duration-fast)"
        >
          <span className="text-n-9">←</span>
          <span className="font-mono text-xs text-n-10 truncate">{openFile}</span>
        </button>
        <FileDiffView repoId={repoId} oid={commit.oid} path={openFile} />
      </div>
    );
  }

  return (
    <div className="text-sm">
      <div className="px-3 py-2 border-b border-n-5">
        <p className="font-mono text-xs text-n-8 break-all">{commit.oid}</p>
        <p className="mt-1 whitespace-pre-wrap text-n-11">{commit.message}</p>
      </div>

      <dl className="px-3 py-2 border-b border-n-5 flex flex-col gap-2">
        <SignatureRow label="autor" signature={commit.author} />
        <SignatureRow label="committer" signature={commit.committer} />
      </dl>

      {commit.parents.length > 0 && (
        <div className="px-3 py-2 border-b border-n-5">
          <p className="text-xs uppercase tracking-[0.1em] text-n-8 mb-1">
            {commit.parents.length > 1 ? "pais" : "pai"}
          </p>
          <div className="flex flex-col gap-0.5">
            {commit.parents.map((parent) => (
              <button
                key={parent}
                type="button"
                onClick={() => select(parent)}
                className="text-left font-mono text-xs text-n-9 hover:text-n-11 transition-colors duration-(--duration-fast)"
              >
                {parent.slice(0, 12)}
              </button>
            ))}
          </div>
        </div>
      )}

      <div className="px-3 py-2 border-b border-n-5 text-xs text-n-9 font-mono">
        {commit.files.length} arquivo{commit.files.length === 1 ? "" : "s"} ·{" "}
        <span className="text-add">+{commit.insertions}</span>{" "}
        <span className="text-del">-{commit.deletions}</span>
      </div>

      <ul>
        {commit.files.map((file) => (
          <FileRow key={file.path} file={file} onOpen={() => setOpenFile(file.path)} />
        ))}
      </ul>
    </div>
  );
}
