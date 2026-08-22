//! Recorte de patch: pegar o diff completo de um arquivo e emitir só os pedaços escolhidos.
//!
//! É o que torna possível stagear um hunk sem stagear o arquivo inteiro. O caminho é o do
//! `BLOCO-E.md`: **montar um patch parcial e entregá-lo ao `git apply --cached`**, nunca
//! reimplementar a aplicação. O git é quem sabe casar contexto, lidar com CRLF, com `\ No
//! newline at end of file` e com os filtros de `.gitattributes`; refazer isso aqui seria fonte
//! garantida de corrupção silenciosa.
//!
//! O texto de entrada é o patch **cru do libgit2** ([`crate::read::RepoRead::worktree_patch`]),
//! não uma reconstrução a partir da `FileDiff` que a UI recebeu: o cabeçalho de verdade carrega
//! `new file mode`, `deleted file mode`, modo de arquivo e os dois blobs, e nada disso cabe na
//! forma que a interface consome. Só o que este módulo precisa entender são os `@@`.

/// Um hunk já separado do texto, com o cabeçalho decomposto.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hunk {
    pub old_start: u32,
    pub old_count: u32,
    pub new_start: u32,
    pub new_count: u32,
    /// O que vem depois do segundo `@@` — o git põe ali a assinatura da função. Preservado
    /// como veio, inclusive o espaço inicial.
    pub section: String,
    /// As linhas do corpo, verbatim e com o marcador (` `, `+`, `-`, `\`) na frente.
    pub lines: Vec<String>,
}

/// Um patch de **um** arquivo: o cabeçalho que o git escreveu e os hunks.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Patch {
    /// Tudo antes do primeiro `@@`, verbatim: `diff --git`, `index`, `new file mode`, `---`,
    /// `+++`. Reescrever qualquer uma dessas linhas seria adivinhar o que o git já disse.
    pub header: Vec<String>,
    pub hunks: Vec<Hunk>,
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PatchError {
    #[error("não consegui interpretar o patch deste arquivo")]
    Malformed,
    #[error("nenhum trecho selecionado")]
    EmptySelection,
    #[error("trecho selecionado não existe neste arquivo")]
    OutOfRange,
}

/// `-1,3` ou `-1`: o git omite a contagem quando ela é 1, e escreve `,0` quando é zero.
fn range(start: u32, count: u32) -> String {
    if count == 1 {
        start.to_string()
    } else {
        format!("{start},{count}")
    }
}

/// `@@ -1,3 +1,4 @@ fn algo()` → os quatro números e a seção.
fn parse_hunk_header(line: &str) -> Option<(u32, u32, u32, u32, String)> {
    let rest = line.strip_prefix("@@ ")?;
    let (ranges, section) = rest.split_once(" @@")?;
    let (old, new) = ranges.split_once(' ')?;

    let numbers = |part: &str, sign: char| -> Option<(u32, u32)> {
        let part = part.strip_prefix(sign)?;
        match part.split_once(',') {
            Some((start, count)) => Some((start.parse().ok()?, count.parse().ok()?)),
            // Contagem omitida é 1 — é assim que o git escreve um hunk de uma linha só.
            None => Some((part.parse().ok()?, 1)),
        }
    };

    let (old_start, old_count) = numbers(old, '-')?;
    let (new_start, new_count) = numbers(new, '+')?;

    Some((
        old_start,
        old_count,
        new_start,
        new_count,
        section.to_owned(),
    ))
}

/// Quebra o patch cru em cabeçalho e hunks.
pub fn parse(raw: &str) -> Result<Patch, PatchError> {
    let mut header = Vec::new();
    let mut hunks: Vec<Hunk> = Vec::new();

    for line in raw.lines() {
        if let Some((old_start, old_count, new_start, new_count, section)) = parse_hunk_header(line)
        {
            hunks.push(Hunk {
                old_start,
                old_count,
                new_start,
                new_count,
                section,
                lines: Vec::new(),
            });
            continue;
        }

        match hunks.last_mut() {
            // Já dentro de um hunk: tudo é corpo, verbatim.
            Some(hunk) => hunk.lines.push(line.to_owned()),
            None => header.push(line.to_owned()),
        }
    }

    if header.is_empty() || hunks.is_empty() {
        return Err(PatchError::Malformed);
    }

    Ok(Patch { header, hunks })
}

/// Um hunk escolhido, inteiro ou por linhas.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HunkSelection {
    pub hunk: usize,
    /// `None` é o hunk inteiro (Passo 50). `Some` são as **linhas de mudança** escolhidas
    /// dentro dele (Passo 51), numeradas **só entre as de mudança** — a primeira `+` ou `-` do
    /// hunk é 0, a segunda é 1, e linhas de contexto não entram na conta.
    ///
    /// Contar só as mudanças, e não todas as linhas do corpo, é o que faz a numeração do
    /// cliente e a daqui coincidirem sempre: o libgit2 emite marcadores de fim-de-arquivo-sem-
    /// quebra-de-linha que aparecem no texto do patch e não na lista que a UI desenha, e uma
    /// numeração sobre "todas as linhas" divergiria exatamente nos arquivos que têm isso.
    /// Selecionar contexto também não significaria nada: contexto não muda de lado.
    pub lines: Option<Vec<usize>>,
}

impl HunkSelection {
    pub fn whole(hunk: usize) -> Self {
        Self { hunk, lines: None }
    }
}

/// O que uma linha do corpo é, decidido pelo primeiro caractere.
enum Body<'a> {
    /// Contexto sai verbatim, então o conteúdo não é lido — só o fato de ser contexto.
    Context,
    Addition(&'a str),
    Deletion(&'a str),
    /// `\ No newline at end of file`. Pertence à linha anterior, não é linha do arquivo.
    NoNewline(&'a str),
}

fn classify(line: &str) -> Body<'_> {
    match line.chars().next() {
        Some('+') => Body::Addition(&line[1..]),
        Some('-') => Body::Deletion(&line[1..]),
        Some('\\') => Body::NoNewline(line),
        // Contexto normal (` texto`) e a linha vazia que alguns geradores escrevem no lugar de
        // um contexto de linha em branco.
        _ => Body::Context,
    }
}

/// Um hunk já reduzido: o corpo emitido e as contagens dos dois lados.
struct Reduced {
    lines: Vec<String>,
    old_count: u32,
    new_count: u32,
    /// Quantas linhas de mudança sobreviveram. Zero significa um hunk que virou só contexto —
    /// não vale a pena emiti-lo.
    changes: usize,
}

/// Aplica a seleção de linhas a um hunk.
///
/// As duas regras que fazem o patch parcial ser correto:
///
/// - uma **adição não escolhida** simplesmente **some** — ela ainda não existe do lado antigo,
///   e não vai passar para o novo;
/// - uma **remoção não escolhida** vira **contexto** — a linha continua nos dois lados, e o
///   `git apply` precisa vê-la para casar o trecho.
///
/// Trocar uma pela outra é o erro clássico de quem monta patch parcial na mão: some com uma
/// linha que devia ficar, ou o patch deixa de casar com o arquivo.
fn reduce(hunk: &Hunk, selected: Option<&[usize]>) -> Result<Reduced, PatchError> {
    if let Some(selected) = selected {
        let total = hunk
            .lines
            .iter()
            .filter(|line| matches!(classify(line), Body::Addition(_) | Body::Deletion(_)))
            .count();

        if selected.iter().any(|index| *index >= total) {
            return Err(PatchError::OutOfRange);
        }
    }

    let chosen = |index: usize| selected.is_none_or(|selected| selected.contains(&index));

    let mut out = Reduced {
        lines: Vec::with_capacity(hunk.lines.len()),
        old_count: 0,
        new_count: 0,
        changes: 0,
    };

    let mut change_index = 0;
    // O marcador de "sem quebra de linha no fim" só faz sentido colado na linha que ele
    // descreve; se essa linha caiu fora, ele cai junto.
    let mut kept_previous = true;

    for line in &hunk.lines {
        match classify(line) {
            Body::Context => {
                out.lines.push(line.clone());
                out.old_count += 1;
                out.new_count += 1;
                kept_previous = true;
            }
            Body::Addition(text) => {
                let keep = chosen(change_index);
                change_index += 1;

                if keep {
                    out.lines.push(format!("+{text}"));
                    out.new_count += 1;
                    out.changes += 1;
                }
                kept_previous = keep;
            }
            Body::Deletion(text) => {
                let keep = chosen(change_index);
                change_index += 1;

                if keep {
                    out.lines.push(format!("-{text}"));
                    out.old_count += 1;
                    out.changes += 1;
                } else {
                    out.lines.push(format!(" {text}"));
                    out.old_count += 1;
                    out.new_count += 1;
                }
                kept_previous = true;
            }
            Body::NoNewline(marker) => {
                if kept_previous {
                    out.lines.push(marker.to_owned());
                }
            }
        }
    }

    Ok(out)
}

impl Patch {
    /// O patch reduzido ao que foi escolhido, pronto para o `git apply`.
    ///
    /// Os índices de hunk são os mesmos que a UI recebeu em `FileDiff` — a ordem dos hunks é a
    /// ordem do arquivo, e as duas vêm do mesmo diff.
    ///
    /// **A linha inicial do lado novo é recalculada.** O `git apply --cached` aplica contra o
    /// conteúdo do índice, que continua sendo o de antes de qualquer hunk; mas o arquivo que sai
    /// tem só o que foi escolhido, então cada hunk mantido precisa levar em conta o deslocamento
    /// acumulado apenas do que **ficou**. Sem isso, stagear o terceiro de três hunks produziria
    /// um patch apontando para uma linha que não existe do lado que ele está construindo.
    pub fn select(&self, selection: &[HunkSelection]) -> Result<String, PatchError> {
        if selection.is_empty() {
            return Err(PatchError::EmptySelection);
        }
        if selection.iter().any(|item| item.hunk >= self.hunks.len()) {
            return Err(PatchError::OutOfRange);
        }

        let mut chosen = selection.to_vec();
        chosen.sort_by_key(|item| item.hunk);

        let mut out = String::new();
        for line in &self.header {
            out.push_str(line);
            out.push('\n');
        }

        // Assinado: um hunk que remove mais do que acrescenta desloca para trás.
        let mut offset: i64 = 0;
        let mut emitted = 0;

        for item in chosen {
            let hunk = &self.hunks[item.hunk];
            let reduced = reduce(hunk, item.lines.as_deref())?;

            // Hunk que virou só contexto não muda nada: emiti-lo seria pedir ao git para
            // aplicar um patch vazio, e um `git apply` de patch sem mudança nenhuma falha.
            if reduced.changes == 0 {
                continue;
            }

            let new_start = (hunk.old_start as i64 + offset).max(0) as u32;

            out.push_str(&format!(
                "@@ -{} +{} @@{}\n",
                range(hunk.old_start, reduced.old_count),
                range(new_start, reduced.new_count),
                hunk.section
            ));

            for line in &reduced.lines {
                out.push_str(line);
                out.push('\n');
            }

            offset += reduced.new_count as i64 - reduced.old_count as i64;
            emitted += 1;
        }

        // Só linhas de contexto escolhidas, ou lista de linhas vazia em todos os hunks: o
        // pedido é bem formado, mas não sobra nada para aplicar.
        if emitted == 0 {
            return Err(PatchError::EmptySelection);
        }

        Ok(out)
    }

    /// Atalho para "estes hunks inteiros" — a forma do Passo 50.
    pub fn select_hunks(&self, hunks: &[usize]) -> Result<String, PatchError> {
        let selection: Vec<_> = hunks.iter().copied().map(HunkSelection::whole).collect();
        self.select(&selection)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Três hunks num arquivo só, com o mesmo formato que o git escreve.
    const TRES_HUNKS: &str = "\
diff --git a/a.txt b/a.txt
index 1111111..2222222 100644
--- a/a.txt
+++ b/a.txt
@@ -1,3 +1,4 @@
 um
+novo-1
 dois
 tres
@@ -10,3 +11,4 @@ fn meio()
 dez
+novo-2
 onze
 doze
@@ -20,3 +22,4 @@
 vinte
+novo-3
 vinte-e-um
 vinte-e-dois
";

    #[test]
    fn separa_cabecalho_dos_hunks() {
        let patch = parse(TRES_HUNKS).unwrap();

        assert_eq!(patch.header.len(), 4);
        assert_eq!(patch.header[0], "diff --git a/a.txt b/a.txt");
        assert_eq!(patch.hunks.len(), 3);

        assert_eq!(patch.hunks[1].old_start, 10);
        assert_eq!(patch.hunks[1].old_count, 3);
        assert_eq!(patch.hunks[1].new_start, 11);
        assert_eq!(patch.hunks[1].new_count, 4);
        // A assinatura da função vem junto, com o espaço que o git pôs.
        assert_eq!(patch.hunks[1].section, " fn meio()");
        // Corpo verbatim, com o marcador na frente — é assim que ele volta para o `git apply`.
        assert_eq!(patch.hunks[1].lines, [" dez", "+novo-2", " onze", " doze"]);
    }

    #[test]
    fn contagem_omitida_vale_um() {
        let patch =
            parse("diff --git a/x b/x\n--- a/x\n+++ b/x\n@@ -1 +1,2 @@\n linha\n+outra\n").unwrap();

        assert_eq!(patch.hunks[0].old_count, 1);
        assert_eq!(patch.hunks[0].new_count, 2);
    }

    #[test]
    fn um_hunk_do_meio_sai_com_o_cabecalho_inteiro_e_so_ele() {
        let saida = parse(TRES_HUNKS).unwrap().select_hunks(&[1]).unwrap();

        assert!(saida.starts_with("diff --git a/a.txt b/a.txt\n"));
        assert!(saida.contains("--- a/a.txt\n+++ b/a.txt\n"));
        assert_eq!(saida.matches("@@ -").count(), 1);
        assert!(saida.contains("+novo-2"));
        assert!(!saida.contains("+novo-1"));
        assert!(!saida.contains("+novo-3"));
    }

    #[test]
    fn o_lado_novo_e_recalculado_quando_hunks_ficam_de_fora() {
        let patch = parse(TRES_HUNKS).unwrap();

        // Sozinho, o terceiro hunk não pode carregar o `+22` que ele tinha no patch inteiro:
        // ali os dois hunks anteriores já tinham empurrado o arquivo duas linhas para baixo, e
        // num patch que não os contém esse deslocamento não existe.
        assert!(patch
            .select_hunks(&[2])
            .unwrap()
            .contains("@@ -20,3 +20,4 @@"));

        // Com o primeiro junto, o deslocamento de uma linha volta a valer.
        let dois = patch.select_hunks(&[0, 2]).unwrap();
        assert!(dois.contains("@@ -1,3 +1,4 @@"), "{dois}");
        assert!(dois.contains("@@ -20,3 +21,4 @@"), "{dois}");
    }

    #[test]
    fn selecionar_todos_reproduz_os_cabecalhos_originais() {
        let saida = parse(TRES_HUNKS).unwrap().select_hunks(&[0, 1, 2]).unwrap();

        assert!(saida.contains("@@ -1,3 +1,4 @@"));
        assert!(saida.contains("@@ -10,3 +11,4 @@ fn meio()"));
        assert!(saida.contains("@@ -20,3 +22,4 @@"));
    }

    #[test]
    fn indice_fora_da_faixa_e_selecao_vazia_sao_recusados() {
        let patch = parse(TRES_HUNKS).unwrap();

        assert_eq!(patch.select_hunks(&[]), Err(PatchError::EmptySelection));
        assert_eq!(patch.select_hunks(&[3]), Err(PatchError::OutOfRange));
    }

    /// Um hunk com duas remoções e duas adições intercaladas com contexto — o mínimo para
    /// exercitar as duas regras do recorte por linha.
    const MISTO: &str = "\
diff --git a/b.txt b/b.txt
index 1111111..2222222 100644
--- a/b.txt
+++ b/b.txt
@@ -1,4 +1,4 @@
 topo
-velha-1
+nova-1
 meio
-velha-2
+nova-2
";

    fn corpo(patch: &str) -> Vec<&str> {
        patch
            .lines()
            .skip_while(|line| !line.starts_with("@@"))
            .collect()
    }

    #[test]
    fn adicao_nao_escolhida_some_e_remocao_nao_escolhida_vira_contexto() {
        let patch = parse(MISTO).unwrap();

        // As linhas de mudança são, em ordem: 0 `-velha-1`, 1 `+nova-1`, 2 `-velha-2`,
        // 3 `+nova-2`. Escolher só o primeiro par é o caso de "stagear a primeira troca".
        let saida = patch
            .select(&[HunkSelection {
                hunk: 0,
                lines: Some(vec![0, 1]),
            }])
            .unwrap();

        assert_eq!(
            corpo(&saida),
            [
                // O lado antigo continua com as 4 linhas dele (nada sumiu); o novo também,
                // porque uma troca entrou e a outra virou contexto nos dois lados.
                "@@ -1,4 +1,4 @@",
                " topo",
                "-velha-1",
                "+nova-1",
                " meio",
                // `velha-2` não foi escolhida para sair: continua nos dois lados, como contexto.
                " velha-2",
            ]
        );
        // E `nova-2` não entra: ela ainda não existe do lado antigo.
        assert!(!saida.contains("nova-2"), "{saida}");
    }

    #[test]
    fn escolher_so_uma_remocao_encolhe_o_lado_novo() {
        let patch = parse(MISTO).unwrap();

        let saida = patch
            .select(&[HunkSelection {
                hunk: 0,
                lines: Some(vec![0]),
            }])
            .unwrap();

        // Uma linha a menos do lado novo, nenhuma a menos do antigo.
        assert_eq!(corpo(&saida)[0], "@@ -1,4 +1,3 @@");
        assert!(saida.contains("-velha-1"), "{saida}");
        assert!(saida.contains(" velha-2"), "{saida}");
    }

    #[test]
    fn so_contexto_escolhido_nao_produz_patch() {
        let patch = parse(MISTO).unwrap();

        // Lista vazia de linhas: nenhuma mudança sobrevive, e um patch só de contexto não é
        // aplicável — o git recusaria um patch que não muda nada.
        assert_eq!(
            patch.select(&[HunkSelection {
                hunk: 0,
                lines: Some(vec![])
            }]),
            Err(PatchError::EmptySelection)
        );
    }

    #[test]
    fn linha_fora_da_faixa_e_recusada() {
        let patch = parse(MISTO).unwrap();

        assert_eq!(
            patch.select(&[HunkSelection {
                hunk: 0,
                lines: Some(vec![4])
            }]),
            Err(PatchError::OutOfRange)
        );
    }

    #[test]
    fn o_marcador_de_fim_sem_quebra_acompanha_a_linha_dele() {
        let raw = "\
diff --git a/c.txt b/c.txt
--- a/c.txt
+++ b/c.txt
@@ -1,2 +1,2 @@
 topo
-fim velho
\\ No newline at end of file
+fim novo
\\ No newline at end of file
";
        let patch = parse(raw).unwrap();

        // Escolhendo as duas, os dois marcadores ficam.
        let ambas = patch
            .select(&[HunkSelection {
                hunk: 0,
                lines: Some(vec![0, 1]),
            }])
            .unwrap();
        assert_eq!(ambas.matches("No newline").count(), 2, "{ambas}");

        // Escolhendo só a remoção, o marcador da adição descartada some junto com ela.
        let so_remocao = patch
            .select(&[HunkSelection {
                hunk: 0,
                lines: Some(vec![0]),
            }])
            .unwrap();
        assert_eq!(so_remocao.matches("No newline").count(), 1, "{so_remocao}");
        assert!(!so_remocao.contains("fim novo"), "{so_remocao}");
    }

    #[test]
    fn texto_que_nao_e_patch_nao_vira_patch() {
        assert_eq!(parse("qualquer coisa\n"), Err(PatchError::Malformed));
        assert_eq!(parse(""), Err(PatchError::Malformed));
    }
}
