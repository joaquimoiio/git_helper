# Fontes embutidas no porcelain

Ambas são **SIL Open Font License 1.1**, que permite redistribuição embutida em software,
inclusive em binário, desde que a licença acompanhe os arquivos e as fontes não sejam
vendidas isoladamente.

| Arquivo | Família | Origem |
|---|---|---|
| `Inter-latin.woff2`, `Inter-latin-ext.woff2` | Inter (variável, 100–900) | https://github.com/rsms/inter |
| `JetBrainsMono-latin.woff2`, `JetBrainsMono-latin-ext.woff2` | JetBrains Mono (variável, 100–800) | https://github.com/JetBrains/JetBrainsMono |

Os arquivos são os subsets `latin` e `latin-ext` distribuídos pelo Google Fonts — o texto
do app é português e as mensagens de commit são majoritariamente latinas, então os subsets
cirílico, grego e vietnamita ficam de fora. Cada família é uma fonte **variável** num único
arquivo por subset: um só download cobre todos os pesos que a UI usa.

Baixados em 2026-08-21. Nenhum request sai da máquina em tempo de execução: o
`@font-face` em `web/src/styles/tokens.css` aponta para estes arquivos, que o Vite copia
para `web/dist` e o `rust-embed` embute no binário.

Texto integral da licença: https://openfontlicense.org/open-font-license-official-text/
