/* Raiz do app: provider do TanStack Query e o portão do handshake.
 *
 * Nada da UI monta antes de haver sessão. Não é zelo estético — é que todo dado da tela
 * vem de `/api/`, e montar o shell primeiro só produziria uma casca cheia de 401. */

import { QueryClient, QueryClientProvider, useQuery } from "@tanstack/react-query";

import { ApiError, establishSession } from "../lib/api";
import { Shell } from "./Shell";

const client = new QueryClient({
  defaultOptions: {
    queries: {
      // O servidor é local e o dado é o repositório do usuário, que muda por fora. Quem
      // avisa de mudança é o WebSocket (Bloco C em diante), não um `refetchInterval`.
      refetchOnWindowFocus: false,
      staleTime: 5_000,
      // Token recusado não melhora na segunda tentativa; erro de rede em loopback também
      // não. Uma tentativa a mais e para.
      retry: (attempt, error) => attempt < 1 && !(error instanceof ApiError),
    },
  },
});

function Curtain({ title, detail }: { title: string; detail: string }) {
  return (
    <div className="h-full flex flex-col items-center justify-center gap-2 bg-n-0 text-n-11">
      <p className="text-md">{title}</p>
      <p className="text-sm font-mono text-n-9 max-w-[52ch] text-center">{detail}</p>
    </div>
  );
}

function Boot() {
  const session = useQuery({
    queryKey: ["session"],
    queryFn: establishSession,
    // O handshake é um efeito colateral que só pode acontecer uma vez: depois do
    // `replaceState` o `?t=` não existe mais para uma segunda tentativa.
    retry: false,
    staleTime: Infinity,
  });

  if (session.isPending) return <Curtain title="porcelain" detail="autenticando…" />;

  if (session.isError) {
    const unauthorized = session.error instanceof ApiError && session.error.status === 401;

    return (
      <Curtain
        title={unauthorized ? "sem sessão" : "não consegui falar com o servidor"}
        detail={
          unauthorized
            ? "esta aba não tem o token deste boot. feche-a e abra o porcelain pelo terminal — o token vale só enquanto o processo estiver vivo."
            : session.error.message
        }
      />
    );
  }

  return <Shell />;
}

export function App() {
  return (
    <QueryClientProvider client={client}>
      <Boot />
    </QueryClientProvider>
  );
}
