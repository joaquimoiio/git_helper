/* A caixa de mensagem, no pé do painel de status.
 *
 * Uma área de texto só, com assunto e corpo juntos — e não dois campos: é a forma que o
 * `commit.template` do usuário tem, é a forma que o git recebe (`-F -`) e é a forma que quem
 * cola uma mensagem pronta de outro lugar espera. A separação entre assunto e corpo é a linha
 * em branco, como sempre foi.
 *
 * A régua de 50/72 é um aviso, nunca uma trava: a convenção é forte, mas é convenção — quem
 * escreve um assunto de 55 caracteres num commit de correção urgente não deve ser impedido.
 */

import { useEffect, useRef, useState, type RefObject } from "react";

import { useCommitDetail } from "../../lib/commit";
import { useCommit, useCommitTemplate } from "../../lib/status";

/** Convenção do git: assunto até 50, corpo até 72 por linha. */
const SUBJECT_LIMIT = 50;
const BODY_LIMIT = 72;

function Ruler({ message }: { message: string }) {
  const [subject, ...body] = message.split("\n");
  const longas = body.filter((line) => line.length > BODY_LIMIT).length;

  return (
    <div className="flex items-baseline gap-3 text-xs font-mono">
      <span className={subject.length > SUBJECT_LIMIT ? "text-conflict" : "text-n-8"}>
        assunto {subject.length}/{SUBJECT_LIMIT}
      </span>
      {longas > 0 && (
        <span className="text-conflict">
          {longas} linha{longas > 1 ? "s" : ""} do corpo acima de {BODY_LIMIT}
        </span>
      )}
      {/* A segunda linha tem que estar em branco: é ela que separa assunto de corpo. Sem isso o
          git trata a mensagem inteira como assunto, e o log fica com uma linha gigante. */}
      {message.split("\n").length > 1 && message.split("\n")[1].trim() !== "" && (
        <span className="text-conflict">falta a linha em branco depois do assunto</span>
      )}
    </div>
  );
}

/** Um interruptor de linha, do tamanho do resto da barra. */
function Toggle({
  on,
  onChange,
  label,
  title,
  disabled,
}: {
  on: boolean;
  onChange: (on: boolean) => void;
  label: string;
  title: string;
  disabled?: boolean;
}) {
  return (
    <button
      type="button"
      title={title}
      disabled={disabled}
      onClick={() => onChange(!on)}
      className={`text-xs px-1.5 py-0.5 rounded-sm border transition-colors duration-(--duration-fast) disabled:text-n-7 ${
        on ? "border-n-7 bg-n-4 text-n-11" : "border-n-6 text-n-9 hover:text-n-11"
      }`}
    >
      {label}
    </button>
  );
}

export function CommitBox({
  repoId,
  canCommit,
  headOid,
  textareaRef,
}: {
  repoId: string;
  /** Há algo preparado. Só desabilita o botão — quem recusa de verdade é o git. */
  canCommit: boolean;
  /** `null` em repositório sem commit nenhum: não há o que emendar ali. */
  headOid: string | null;
  textareaRef: RefObject<HTMLTextAreaElement | null>;
}) {
  const [message, setMessage] = useState("");
  const [amend, setAmend] = useState(false);
  const [signoff, setSignoff] = useState(false);
  // `null` é "como o usuário configurou" — o padrão, e o certo. Os dois outros valores forçam,
  // só para este commit.
  const [gpgSign, setGpgSign] = useState<boolean | null>(null);

  const commit = useCommit(repoId);
  const template = useCommitTemplate(repoId).data?.template ?? null;
  // Só busca quando o amend está ligado: é a mensagem que vai ser reescrita.
  const anterior = useCommitDetail(repoId, amend ? headOid : null).data?.message ?? null;

  // O template entra **uma vez**, e só numa caixa vazia: recarregá-lo por cima de algo que a
  // pessoa está escrevendo seria apagar o trabalho dela.
  const semeado = useRef(false);
  useEffect(() => {
    if (semeado.current || template === null) return;
    semeado.current = true;
    setMessage((atual) => (atual === "" ? template : atual));
  }, [template]);

  // Mesma regra para a mensagem anterior ao ligar o amend: ela entra na caixa vazia ou por cima
  // do template intocado, nunca por cima do que alguém escreveu.
  //
  // Ajuste **durante o render**, e não num efeito: a mensagem anterior chega de uma consulta,
  // então não dá para tratá-la no evento do clique — e um efeito que chama `setState` aqui só
  // provoca um segundo render para o mesmo resultado.
  const [carregadoDe, setCarregadoDe] = useState<string | null>(null);
  if (amend && anterior !== null && carregadoDe !== headOid) {
    setCarregadoDe(headOid);
    if (message.trim() === "" || message === template) setMessage(anterior);
  }

  function submit() {
    if (message.trim() === "" || commit.isPending) return;

    commit.mutate(
      { message, amend, signoff, gpgSign: gpgSign ?? undefined },
      {
        // Só limpa quando deu certo: uma mensagem longa não pode sumir porque um hook recusou.
        onSuccess: () => {
          setMessage("");
          setAmend(false);
        },
      },
    );
  }

  function onKeyDown(event: React.KeyboardEvent) {
    // Ctrl/Cmd+Enter: fora da lista que o navegador rouba, e a convenção de "enviar" em toda
    // caixa de texto multilinha que existe.
    if ((event.ctrlKey || event.metaKey) && event.key === "Enter") {
      event.preventDefault();
      submit();
    }
  }

  return (
    <div className="shrink-0 border-t border-n-5 bg-n-1">
      {commit.isError && (
        <p className="px-3 py-1 text-sm text-error bg-error-bg">{commit.error.message}</p>
      )}
      {commit.isSuccess && (
        <p className="px-3 py-1 text-sm text-n-9">
          commit <span className="font-mono text-n-11">{commit.data.oid.slice(0, 7)}</span> criado.
        </p>
      )}
      <textarea
        ref={textareaRef}
        value={message}
        onChange={(event) => setMessage(event.target.value)}
        onKeyDown={onKeyDown}
        rows={4}
        spellCheck
        placeholder="assunto na primeira linha, linha em branco, corpo"
        className="w-full px-3 py-2 bg-transparent text-sm font-mono text-n-11 placeholder:text-n-8 outline-none resize-none"
      />
      <div className="flex items-center gap-2 px-3 pb-1">
        <Toggle
          on={amend}
          onChange={(on) => {
            setAmend(on);
            // Desligar reabre a porta para carregar a mensagem anterior de novo se alguém
            // religar — sem isto, ligar-desligar-ligar deixaria a caixa como estava.
            if (!on) setCarregadoDe(null);
          }}
          disabled={headOid === null}
          label="emendar"
          title="reescreve o commit anterior em vez de criar um novo (--amend)"
        />
        <Toggle
          on={signoff}
          onChange={setSignoff}
          label="signoff"
          title="acrescenta a linha Signed-off-by com a identidade configurada"
        />
        <Toggle
          on={gpgSign === true}
          // Terceiro estado de propósito: desligar o botão volta para "como você configurou",
          // não para "não assinar". Quem tem `commit.gpgSign = true` continua assinando.
          onChange={(on) => setGpgSign(on ? true : null)}
          label="assinar"
          title="força a assinatura GPG deste commit; desligado, vale o seu commit.gpgSign"
        />
      </div>
      <div className="flex items-center justify-between gap-3 px-3 pb-2">
        <Ruler message={message} />
        <div className="flex items-center gap-2">
          <span className="text-xs font-mono text-n-8">ctrl+enter</span>
          <button
            type="button"
            onClick={submit}
            // Com `--amend` não é preciso haver nada preparado: emendar só a mensagem é um uso
            // legítimo, e o mais comum de todos.
            disabled={message.trim() === "" || (!canCommit && !amend) || commit.isPending}
            className="text-xs px-2 py-0.5 rounded-sm border border-n-6 text-n-10 hover:text-n-11 disabled:text-n-7 disabled:border-n-6 transition-colors duration-(--duration-fast)"
          >
            commitar
          </button>
        </div>
      </div>
    </div>
  );
}
