/* O pedido de senha do ssh, na interface.
 *
 * Aparece acima do painel de jobs porque bloqueia: enquanto ninguém responde, o clone está
 * parado. O texto do prompt é o do ssh, sem tradução — ele nomeia a chave, e é por ele que o
 * usuário sabe qual passphrase digitar.
 *
 * O campo é `type="password"` e `autoComplete="off"`: não há por que o gerenciador de senhas do
 * navegador guardar a passphrase de uma chave SSH.
 */

import { useState } from "react";

import type { OpenPrompt } from "../../lib/askpass";
import { useAnswerAskpass, usePendingPrompt } from "../../lib/askpass";
import { useCancelJob } from "../../lib/jobs";

/**
 * Um prompt novo (passphrase errada na primeira tentativa) chega com outro `promptId`. A `key`
 * remonta o formulário, o que zera o campo e refoca sem nenhum efeito para sincronizar estado —
 * é o que separa "tente outra vez" de "seu texto continua aí".
 */
export function AskpassPrompt() {
  const prompt = usePendingPrompt();

  if (!prompt) return null;

  return <PromptForm key={prompt.promptId} prompt={prompt} />;
}

function PromptForm({ prompt }: { prompt: OpenPrompt }) {
  const answer = useAnswerAskpass();
  const cancel = useCancelJob();
  const [secret, setSecret] = useState("");

  function submit(event: React.FormEvent) {
    event.preventDefault();
    if (!secret) return;

    answer.mutate({ ...prompt, secret });
    // Some da memória do componente no mesmo instante em que sai para o servidor.
    setSecret("");
  }

  return (
    <form
      onSubmit={submit}
      className="shrink-0 border-t border-n-5 bg-n-2 px-3 py-2"
      onKeyDown={(event) => {
        if (event.key !== "Escape") return;
        // Fechar sem responder deixaria o ssh esperando. Desistir aqui é desistir do clone.
        event.preventDefault();
        cancel.mutate(prompt.jobId);
      }}
    >
      <p className="text-sm font-mono text-n-10 break-all">{prompt.prompt}</p>

      <div className="mt-1 flex items-center gap-2">
        <input
          autoFocus
          type="password"
          autoComplete="off"
          value={secret}
          onChange={(event) => setSecret(event.target.value)}
          placeholder="senha"
          className="flex-1 min-w-0 px-2 py-0.5 bg-n-0 border border-n-6 rounded-sm text-base text-n-11 placeholder:text-n-8 outline-none focus-visible:border-n-8"
        />
        <button
          type="submit"
          disabled={!secret || answer.isPending}
          className="px-2 py-0.5 border border-n-6 rounded-sm text-sm text-n-11 hover:bg-n-3 disabled:text-n-8 disabled:hover:bg-transparent transition-colors duration-(--duration-fast)"
        >
          responder
        </button>
        <button
          type="button"
          onClick={() => cancel.mutate(prompt.jobId)}
          className="text-sm text-n-9 hover:text-n-11 transition-colors duration-(--duration-fast)"
        >
          desistir
        </button>
      </div>

      <p className="mt-1 text-xs text-n-8">
        a senha vai direto para o git, em memória. não é gravada em disco nem aparece em `ps`.
      </p>
    </form>
  );
}
