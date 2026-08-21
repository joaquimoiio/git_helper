---
description: Executa TODOS os passos pendentes de um bloco do porcelain, sem parar entre eles
argument-hint: A | B | C | D | E | F | G | H | I
---

Você vai executar **o bloco `$ARGUMENTS` inteiro**, do primeiro passo pendente até o
último, sem devolver o controle ao usuário entre os passos.

Este comando **substitui** a regra 1 do `CLAUDE.md` ("um passo por vez") e a regra 7
("fim de bloco: parar"). Todas as outras regras continuam valendo, em especial a 4:
**cada passo termina com o projeto compilando e rodando.**

## 1. Contexto

1. Leia `CLAUDE.md`.
2. Leia `PROGRESSO.md`.
3. Leia `.claude/blocos/BLOCO-$ARGUMENTS.md`.

Não leia os outros arquivos de bloco.

Se `PROGRESSO.md` indicar outro bloco atual, avise e pergunte antes de executar.

## 2. Pré-voo (obrigatório, antes do primeiro passo)

Verifique que a toolchain necessária para os critérios de aceite deste bloco existe:

- blocos com Rust: `cargo --version`
- blocos com frontend: `node --version` e `npm --version`

Se faltar alguma, **pare aqui** e diga qual comando o usuário precisa rodar. Não escreva
código de 12 passos que você não consegue compilar.

## 3. Laço de execução

Para cada passo pendente, na ordem do bloco:

1. Leia os arquivos que o passo vai tocar. Não reescreva o que já está certo.
2. Escreva as alterações **em disco**, com Write/Edit. Não cole o conteúdo dos arquivos
   na resposta — o usuário lê o diff no editor.
3. Rode o critério de aceite do passo (o `cargo build`, o `curl`, o que o bloco definir).
4. Verde → siga para o próximo passo.
   Vermelho → conserte e rode de novo. **Duas tentativas.** Falhou nas duas, pare o laço
   e relate: o que quebrou, a saída do comando e o que você tentou.
5. Atualize `PROGRESSO.md` (passo concluído, próximo passo, decisões novas com data
   absoluta, pendências). Atualizar a cada passo é o que permite retomar se o laço
   travar no meio.

Saída por passo na resposta: **uma linha só**, no formato

```
Passo N — <nome curto> · <arquivos tocados> · aceite: <comando> ✓
```

Nada de código, nada de bloco de comandos para copiar, nada de "próximo passo será".

## 4. Quando parar antes do fim

Pare o laço e devolva o controle se:

- um critério de aceite falhar duas vezes;
- o passo exigir uma decisão que o bloco não fixa (nome de rota, formato de payload,
  escolha de crate que o `CLAUDE.md` não decidiu) — pergunte, não invente;
- o passo depender de algo interativo na máquina do usuário (login, instalação, senha);
- o passo quiser tocar um arquivo fora do escopo do bloco.

Em qualquer desses casos: diga em qual passo parou, por quê, e o que falta.

## 5. Fechamento

Terminado o último passo:

- atualize a tabela de blocos em `PROGRESSO.md` (bloco → concluído, próximo → em andamento);
- rode uma vez o critério de aceite **do bloco** ("Ao fim do bloco:" no `BLOCO-X.md`) e
  relate o resultado real, não o esperado;
- resuma em no máximo 5 linhas o que o bloco entregou e o que ficou pendente;
- diga: *"bloco concluído, pode dar `/clear` e rodar `/blocao <próximo>`"*.

E pare. Não comece o bloco seguinte.
