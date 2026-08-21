/* Espelho dos tipos serde do `porc-server`. Arquivo único e escrito à mão: enquanto a API
 * couber numa tela, gerar isto de um schema custa mais manutenção do que resolve.
 *
 * O Rust nomeia em `snake_case` e serializa em `camelCase`
 * (`#[serde(rename_all = "camelCase")]`), então é o `camelCase` que aparece aqui. */

/** `GET /health` — rota pública, é o que a segunda instância consulta. */
export interface Health {
  name: string;
  version: string;
  pid: number;
  instanceId: string;
}

/** `POST /api/v1/session` — troca o token de boot pelos cookies. */
export interface SessionRequest {
  token: string;
}

export interface SessionResponse {
  ok: boolean;
}

/** `GET /api/v1/whoami` — provisória; vira informação de repositório no Bloco C. */
export interface WhoAmI {
  authenticated: boolean;
  pid: number;
  version: string;
}

export interface Ok {
  ok: boolean;
}
