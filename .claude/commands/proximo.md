---
description: Continua o projeto porcelain do ponto indicado no PROGRESSO.md
argument-hint: (sem argumento)
---

Continue o projeto porcelain exatamente de onde o `PROGRESSO.md` indica.

## 1. Carregue o contexto, nesta ordem

1. Leia `CLAUDE.md`.
2. Leia `PROGRESSO.md` e descubra o **bloco atual** e o **próximo passo**.
3. Leia **apenas** `.claude/blocos/BLOCO-<bloco atual>.md`.

Não leia os outros arquivos de bloco.

Se o bloco atual já estiver concluído, **não avance sozinho** para o próximo bloco: diga
qual é o próximo e peça ao usuário para dar `/clear` e rodar `/bloco <próximo>`.

## 2. Execute apenas o próximo passo

Antes de escrever, olhe o código que já existe nos arquivos que o passo vai tocar.

Regras (detalhe em `CLAUDE.md`):

- **Um passo só.** Nunca emende o seguinte na mesma resposta.
- Máximo 3-4 arquivos; se passar disso, quebre em dois e execute o primeiro.
- Arquivo alterado é mostrado **inteiro**, nunca em patch com reticências.
- O projeto compila e roda ao fim do passo.
- Comandos exatos, prontos para copiar.
- Ideia fora de escopo vira uma linha em "Pendências / ideias", não código.

Formato obrigatório:

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

## 3. Atualize `PROGRESSO.md` e pare

Atualize último passo concluído, próximo passo, tabela de passos, decisões novas (com
data absoluta) e pendências. Se foi o último passo do bloco, diga:

> bloco concluído, pode dar `/clear` e rodar `/bloco <próximo>`

E pare.
