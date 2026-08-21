/* Cliente da API. Todo request do app passa por aqui — é o único lugar que sabe do cookie
 * de CSRF, do prefixo `/api/v1` e do formato de erro. */

import type { SessionResponse, WhoAmI } from "./api-types";

const BASE = "/api/v1";

/** Legível por JS de propósito: é assim que o double-submit funciona. */
const CSRF_COOKIE = "porc_csrf";

export class ApiError extends Error {
  // Campo declarado e atribuído no corpo: `erasableSyntaxOnly` proíbe parâmetro de
  // construtor com modificador, que é sintaxe que o TS *emite*, não só apaga.
  readonly status: number;

  constructor(status: number, message: string) {
    super(message);
    this.name = "ApiError";
    this.status = status;
  }
}

function csrfToken(): string | null {
  const found = document.cookie
    .split(";")
    .map((pair) => pair.trim().split("="))
    .find(([name]) => name === CSRF_COOKIE);

  return found?.[1] ?? null;
}

async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const method = init?.method ?? "GET";
  const headers = new Headers(init?.headers);

  // GET e HEAD não mudam estado e o servidor não exige o header neles.
  if (method !== "GET" && method !== "HEAD") {
    const token = csrfToken();
    if (token) headers.set("X-CSRF-Token", token);
  }
  if (init?.body) headers.set("Content-Type", "application/json");

  const response = await fetch(`${BASE}${path}`, {
    ...init,
    headers,
    // Loopback e mesma origem sempre; nunca mandar credencial para outro lugar.
    credentials: "same-origin",
  });

  if (!response.ok) {
    // O servidor devolve texto legível em erro, não JSON de stack. Se vier vazio, o
    // status ainda diz alguma coisa.
    const detail = (await response.text()).trim();
    throw new ApiError(response.status, detail || `${method} ${path} falhou`);
  }

  return (await response.json()) as T;
}

export const api = {
  get: <T>(path: string) => request<T>(path),
  post: <T>(path: string, body?: unknown) =>
    request<T>(path, { method: "POST", body: body === undefined ? undefined : JSON.stringify(body) }),
};

/**
 * Handshake do boot.
 *
 * O `?t=` chega uma única vez, na URL que o `porc` mandou o navegador abrir. Trocado por
 * cookie, ele é apagado da barra de endereços com `replaceState` — some do histórico e do
 * `Referer`, que é o motivo de o servidor entregá-lo assim.
 *
 * Recarregar a página depois disso não tem token nenhum: quem responde é o cookie. Por
 * isso o `whoami` no fim vale para os dois caminhos.
 */
export async function establishSession(): Promise<WhoAmI> {
  const url = new URL(window.location.href);
  const token = url.searchParams.get("t");

  if (token) {
    await api.post<SessionResponse>("/session", { token });

    url.searchParams.delete("t");
    window.history.replaceState(null, "", `${url.pathname}${url.search}${url.hash}`);
  }

  return api.get<WhoAmI>("/whoami");
}
