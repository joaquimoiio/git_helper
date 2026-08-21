---
description: Executa o próximo passo pendente de um bloco do projeto porcelain
argument-hint: A | B | C | D | E | F | G | H | I
---

Você vai executar **um único passo** do bloco `$ARGUMENTS` do projeto porcelain.

## 1. Carregue o contexto, nesta ordem

1. Leia `CLAUDE.md` — contexto permanente, stack, regras de segurança, design tokens,
   convenções e o formato obrigatório de entrega.
2. Leia `PROGRESSO.md` — onde paramos, decisões já tomadas, pendências.
3. Leia `.claude/blocos/BLOCO-$ARGUMENTS.md` — a especificação **deste** bloco.

**Não leia os outros arquivos de bloco.** Eles existem justamente para não carregar a
especificação inteira do projeto de uma vez.

## 2. Identifique o passo

Compare a lista de passos de `BLOCO-$ARGUMENTS.md` com os passos já concluídos em
`PROGRESSO.md`. O passo a executar é o **primeiro pendente** do bloco.

- Se `PROGRESSO.md` indicar que estamos noutro bloco, avise o usuário e pergunte se ele
  quer mesmo pular — não execute por conta própria.
- Se todos os passos do bloco já estiverem concluídos, diga isso e indique o bloco
  seguinte. Não continue.

## 3. Execute apenas esse passo

Antes de escrever qualquer coisa, olhe o código que já existe nos arquivos que o passo
vai tocar. Não reescreva o que já está certo.

Regras que valem sempre (detalhe em `CLAUDE.md`):

- **Um passo só.** Nunca emende o próximo na mesma resposta.
- Máximo 3-4 arquivos. Se passar disso, quebre em dois passos, execute o primeiro e
  registre a quebra em `PROGRESSO.md`.
- Ao alterar arquivo existente, mostre o **arquivo inteiro atualizado**. Nunca
  `// resto igual`, nunca trecho solto sem caminho.
- O projeto tem que **compilar e rodar** ao fim do passo.
- Comandos exatos, prontos para copiar. Nunca "instale as dependências necessárias".
- Ideia boa fora do escopo vira uma linha em "Pendências / ideias" no `PROGRESSO.md`,
  não código.

Use exatamente este formato:

```
## Passo N — <nome curto>

**O que este passo faz:** uma frase.

**Arquivos:** lista dos arquivos criados ou alterados.

<código completo de cada arquivo, com o caminho no topo>

**Comando para rodar:**
<comandos exatos, em bloco de código, na ordem>

**Como sei que funcionou:** o que aparece na tela ou no terminal.

**Próximo passo será:** uma frase.
```

## 4. Atualize `PROGRESSO.md` e pare

Antes de encerrar a resposta, atualize `PROGRESSO.md`:

- bloco atual, último passo concluído, próximo passo
- linha na tabela de passos concluídos
- qualquer decisão nova que valha para janelas futuras (com data absoluta)
- pendências que surgiram

Se este era o **último passo do bloco**, atualize também a tabela de blocos e diga
explicitamente:

> bloco concluído, pode dar `/clear` e rodar `/bloco <próximo>`

E **pare**. Não comece o bloco seguinte na mesma janela.
