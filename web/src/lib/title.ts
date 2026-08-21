/* Título da aba.
 *
 * O app roda numa aba entre outras vinte. Quem tem três repositórios abertos precisa
 * distinguir os três pelo texto truncado da aba — daí o repositório vir primeiro e o nome
 * do produto por último, que é a parte que o navegador corta antes. */

import { useEffect } from "react";

const APP = "porcelain";

export function documentTitle(repo: string | null, branch: string | null): string {
  if (!repo) return APP;
  if (!branch) return `${repo} — ${APP}`;
  return `${repo} · ${branch} — ${APP}`;
}

export function useDocumentTitle(repo: string | null, branch: string | null) {
  useEffect(() => {
    document.title = documentTitle(repo, branch);
  }, [repo, branch]);
}
