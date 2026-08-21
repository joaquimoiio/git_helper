/* O WebSocket do cliente. Um só, multiplexado por assunto.
 *
 * Reconectar é o caso normal, não a exceção: o laptop dorme, o servidor reinicia, a aba fica em
 * segundo plano. Por isso o socket é um objeto de módulo com backoff próprio, e não um
 * `useEffect` que sobe e desce junto com a árvore de componentes.
 *
 * E reconectar **não** tenta recuperar o que passou. O que se perdeu se recupera por HTTP
 * (`GET /api/v1/jobs/{id}`), que é a razão de aquela rota devolver o estado completo. Um socket
 * que tentasse remontar histórico seria um segundo banco de dados, pior que o primeiro.
 */

import type { ServerMessage } from "./api-types";

type Listener = (message: ServerMessage) => void;

/** Backoff: rápido no primeiro tropeço (servidor reiniciando em dev), calmo depois. */
const BACKOFF_MIN = 250;
const BACKOFF_MAX = 5_000;

const URL = () => `${location.protocol === "https:" ? "wss" : "ws"}://${location.host}/api/v1/ws`;

class Channel {
  private socket: WebSocket | null = null;
  private listeners = new Set<Listener>();
  private topics = new Set<string>();
  private backoff = BACKOFF_MIN;
  private retry: ReturnType<typeof setTimeout> | null = null;
  /** Avisado a cada conexão nova — é o gancho de "pode ter perdido coisa, recarregue". */
  private onOpen = new Set<() => void>();

  connect() {
    if (this.socket && this.socket.readyState !== WebSocket.CLOSED) return;

    const socket = new WebSocket(URL());
    this.socket = socket;

    socket.onopen = () => {
      this.backoff = BACKOFF_MIN;
      // As assinaturas são do cliente, não da conexão: reenviá-las é o que faz uma reconexão
      // ser invisível para quem usa o canal.
      if (this.topics.size > 0) {
        socket.send(JSON.stringify({ type: "subscribe", topics: [...this.topics] }));
      }
      for (const handler of this.onOpen) handler();
    };

    socket.onmessage = (event) => {
      let message: ServerMessage;
      try {
        message = JSON.parse(event.data as string) as ServerMessage;
      } catch {
        // Mensagem que não é JSON não existe no protocolo; ignorar é melhor que derrubar.
        return;
      }
      for (const listener of this.listeners) listener(message);
    };

    socket.onclose = () => {
      this.socket = null;
      this.scheduleRetry();
    };

    // `onerror` sempre é seguido de `onclose`, que é quem agenda a reconexão.
    socket.onerror = () => socket.close();
  }

  private scheduleRetry() {
    if (this.retry !== null) return;

    this.retry = setTimeout(() => {
      this.retry = null;
      this.connect();
    }, this.backoff);

    this.backoff = Math.min(BACKOFF_MAX, this.backoff * 2);
  }

  subscribe(topic: string) {
    this.topics.add(topic);
    if (this.socket?.readyState === WebSocket.OPEN) {
      this.socket.send(JSON.stringify({ type: "subscribe", topics: [topic] }));
    }
  }

  /** Devolve o cancelador, para o componente se desligar ao sair. */
  listen(listener: Listener): () => void {
    this.listeners.add(listener);
    return () => this.listeners.delete(listener);
  }

  whenOpen(handler: () => void): () => void {
    this.onOpen.add(handler);
    return () => this.onOpen.delete(handler);
  }
}

export const channel = new Channel();
