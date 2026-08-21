//! Parser do `--progress` do git.
//!
//! O detalhe que faz toda a diferença: **o git separa as atualizações com `\r`, não com `\n`**.
//! Ele reescreve a mesma linha do terminal dezenas de vezes por segundo. Quem ler esse stderr
//! com um `BufReader::lines()` fica sem nenhum evento até o fim da fase inteira — e aí recebe um
//! "linha" de quinze mil caracteres. Daí o [`Splitter`], que corta em `\r` **e** `\n` e guarda o
//! pedaço incompleto entre um chunk e o outro.
//!
//! O segundo detalhe é que o git alinha os campos com espaços à direita (`"...done.        "`) e
//! prefixa com `remote: ` o que veio do servidor. As duas coisas somem antes do parse.
//!
//! As amostras dos testes são stderr **real**, capturado de clones de verdade.

/// Uma fase do clone, já tipada.
#[derive(Debug, Clone, PartialEq)]
pub enum ProgressEvent {
    /// `remote: Enumerating objects: 3986, done.` — não tem percentual, só o total corrente.
    Enumerating {
        objects: u64,
        done: bool,
    },
    Counting(Counted),
    Compressing(Counted),
    Receiving(Receiving),
    ResolvingDeltas(Counted),
    /// `Updating files:  45% (123/271)` — o checkout depois do fetch.
    CheckingOut(Counted),
    /// Qualquer outra coisa: `Cloning into '…'`, `remote: Total 3986 (delta 572)…`, avisos.
    /// Vira linha de log, não progresso.
    Other(String),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Counted {
    pub percent: u8,
    pub current: u64,
    pub total: u64,
    pub done: bool,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Receiving {
    pub percent: u8,
    pub objects: u64,
    pub total: u64,
    /// Bytes recebidos. `None` no começo, antes de o git ter o que medir.
    pub bytes: Option<u64>,
    /// Bytes por segundo, como o git os calculou. Refazer a conta aqui daria um número
    /// diferente do que o usuário vê no terminal dele, o que não ajuda ninguém.
    pub bytes_per_s: Option<u64>,
    pub done: bool,
}

impl ProgressEvent {
    /// Fase legível, para a UI.
    pub fn phase(&self) -> &'static str {
        match self {
            ProgressEvent::Enumerating { .. } => "enumerando objetos",
            ProgressEvent::Counting(_) => "contando objetos",
            ProgressEvent::Compressing(_) => "comprimindo objetos",
            ProgressEvent::Receiving(_) => "recebendo objetos",
            ProgressEvent::ResolvingDeltas(_) => "resolvendo deltas",
            ProgressEvent::CheckingOut(_) => "escrevendo arquivos",
            ProgressEvent::Other(_) => "",
        }
    }

    /// 0.0–1.0, ou `None` quando a fase não tem total conhecido.
    ///
    /// `Enumerating` é o caso: o git conta objetos sem saber quantos serão. Uma UI que
    /// inventasse uma fração aqui mentiria durante a parte mais lenta de um clone grande.
    pub fn fraction(&self) -> Option<f32> {
        match self {
            ProgressEvent::Enumerating { .. } | ProgressEvent::Other(_) => None,
            ProgressEvent::Counting(counted)
            | ProgressEvent::Compressing(counted)
            | ProgressEvent::ResolvingDeltas(counted)
            | ProgressEvent::CheckingOut(counted) => Some(counted.percent as f32 / 100.0),
            ProgressEvent::Receiving(receiving) => Some(receiving.percent as f32 / 100.0),
        }
    }

    /// Uma linha curta com os números crus.
    pub fn detail(&self) -> Option<String> {
        match self {
            ProgressEvent::Other(_) => None,
            ProgressEvent::Enumerating { objects, .. } => Some(format!("{objects} objetos")),
            ProgressEvent::Counting(counted)
            | ProgressEvent::Compressing(counted)
            | ProgressEvent::ResolvingDeltas(counted)
            | ProgressEvent::CheckingOut(counted) => {
                Some(format!("{}/{}", counted.current, counted.total))
            }
            ProgressEvent::Receiving(receiving) => {
                let mut detail = format!("{}/{}", receiving.objects, receiving.total);
                if let Some(bytes) = receiving.bytes {
                    detail.push_str(&format!(" · {}", human_bytes(bytes)));
                }
                if let Some(rate) = receiving.bytes_per_s {
                    detail.push_str(&format!(" · {}/s", human_bytes(rate)));
                }
                Some(detail)
            }
        }
    }
}

/// Corta o stderr em atualizações, em `\r` **e** `\n`, guardando o resto entre chunks.
///
/// É a peça que separa "eu leio o que o git escreve" de "eu vejo o clone andar".
#[derive(Debug, Default)]
pub struct Splitter {
    buffer: String,
}

impl Splitter {
    /// Consome um pedaço do stderr e devolve as atualizações completas que ele fechou.
    ///
    /// `from_utf8_lossy`: caminho com nome não-UTF-8 pode aparecer numa mensagem, e um clone não
    /// pode falhar por causa disso. O que se perde é um caractere numa linha de log.
    pub fn push(&mut self, chunk: &[u8]) -> Vec<String> {
        self.buffer.push_str(&String::from_utf8_lossy(chunk));

        let mut updates = Vec::new();
        // Um chunk pode fechar várias atualizações de uma vez — leitura de socket não respeita
        // fronteira de linha.
        while let Some(at) = self.buffer.find(['\r', '\n']) {
            let update = self.buffer[..at].to_owned();
            // `at + 1` e não `at + len('\r\n')`: um `\r\n` vira duas fatias, e a segunda é vazia
            // e descartada abaixo. Tratar o par como um só custaria um estado a mais para nada.
            self.buffer.drain(..=at);

            if !update.trim().is_empty() {
                updates.push(update);
            }
        }

        updates
    }

    /// O que sobrou quando o processo terminou sem fechar a última linha.
    pub fn flush(&mut self) -> Option<String> {
        let rest = std::mem::take(&mut self.buffer);

        (!rest.trim().is_empty()).then_some(rest)
    }
}

/// Interpreta uma atualização já isolada pelo [`Splitter`].
pub fn parse(update: &str) -> ProgressEvent {
    // O git preenche a linha com espaços para apagar o que estava escrito antes; e `remote: `
    // marca o que veio do outro lado, o que não muda o significado da fase.
    let line = update.trim();
    let line = line.strip_prefix("remote: ").unwrap_or(line).trim_end();

    if let Some(rest) = line.strip_prefix("Enumerating objects: ") {
        // `3986, done.` ou só `3986`
        let done = rest.contains("done.");
        let objects = rest
            .split(',')
            .next()
            .and_then(|count| count.trim().parse().ok());

        if let Some(objects) = objects {
            return ProgressEvent::Enumerating { objects, done };
        }
    }

    for (prefix, build) in [
        (
            "Counting objects:",
            ProgressEvent::Counting as fn(Counted) -> ProgressEvent,
        ),
        ("Compressing objects:", ProgressEvent::Compressing),
        ("Resolving deltas:", ProgressEvent::ResolvingDeltas),
        ("Updating files:", ProgressEvent::CheckingOut),
    ] {
        if let Some(rest) = line.strip_prefix(prefix) {
            if let Some((counted, _)) = parse_counted(rest) {
                return build(counted);
            }
        }
    }

    if let Some(rest) = line.strip_prefix("Receiving objects:") {
        if let Some((counted, tail)) = parse_counted(rest) {
            let (bytes, bytes_per_s) = parse_throughput(tail);

            return ProgressEvent::Receiving(Receiving {
                percent: counted.percent,
                objects: counted.current,
                total: counted.total,
                bytes,
                bytes_per_s,
                done: counted.done,
            });
        }
    }

    ProgressEvent::Other(line.to_owned())
}

/// `  34% (16766/49311), 7.44 MiB | 14.69 MiB/s` → o contador e o que sobrou depois do `)`.
fn parse_counted(rest: &str) -> Option<(Counted, &str)> {
    let (percent, rest) = rest.split_once('%')?;
    let percent: u8 = percent.trim().parse().ok()?;

    let open = rest.find('(')?;
    let close = rest.find(')')?;
    let (current, total) = rest.get(open + 1..close)?.split_once('/')?;

    let tail = rest.get(close + 1..).unwrap_or("");

    Some((
        Counted {
            percent,
            current: current.trim().parse().ok()?,
            total: total.trim().parse().ok()?,
            done: tail.contains("done."),
        },
        tail,
    ))
}

/// `, 7.44 MiB | 14.69 MiB/s, done.` → `(bytes, bytes por segundo)`.
fn parse_throughput(tail: &str) -> (Option<u64>, Option<u64>) {
    let Some((bytes, rate)) = tail.split_once('|') else {
        return (None, None);
    };

    // Antes da barra sobra `, 7.44 MiB `; depois, `14.69 MiB/s, done.`
    let bytes = bytes.trim_start_matches(',').trim();
    let rate = rate.split(',').next().unwrap_or("").trim();

    (
        parse_size(bytes),
        parse_size(rate.strip_suffix("/s").unwrap_or(rate)),
    )
}

/// `7.44 MiB` → bytes. As unidades são as que o git usa: binárias, com `i`.
fn parse_size(raw: &str) -> Option<u64> {
    let (value, unit) = raw.split_once(char::is_whitespace)?;
    let value: f64 = value.trim().parse().ok()?;

    let scale: u64 = match unit.trim() {
        "B" | "bytes" => 1,
        "KiB" => 1 << 10,
        "MiB" => 1 << 20,
        "GiB" => 1 << 30,
        "TiB" => 1u64 << 40,
        _ => return None,
    };

    Some((value * scale as f64) as u64)
}

/// Inverso do [`parse_size`], para a UI mostrar o mesmo vocabulário do terminal.
fn human_bytes(bytes: u64) -> String {
    const UNITS: [(&str, u64); 4] = [
        ("GiB", 1 << 30),
        ("MiB", 1 << 20),
        ("KiB", 1 << 10),
        ("B", 1),
    ];

    for (unit, scale) in UNITS {
        if bytes >= scale {
            return if scale == 1 {
                format!("{bytes} B")
            } else {
                format!("{:.2} {unit}", bytes as f64 / scale as f64)
            };
        }
    }

    "0 B".to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Trecho **real** do stderr de `git clone --progress https://github.com/rust-lang/log.git`,
    /// com os `\r` e os espaços de preenchimento exatamente como o git os escreveu.
    const REAL: &str = "Cloning into 'progress-probe'...\n\
        remote: Enumerating objects: 3986, done.        \n\
        remote: Counting objects:   0% (1/752)        \r\
        remote: Counting objects: 100% (752/752), done.        \r\
        remote: Compressing objects:  10% (29/290)        \r\
        remote: Compressing objects: 100% (290/290), done.        \r\
        Receiving objects:   0% (1/3986)\r\
        remote: Total 3986 (delta 572), reused 492 (delta 462), pack-reused 3234 (from 4)        \n\
        Receiving objects: 100% (3986/3986), 1.11 MiB | 6.34 MiB/s, done.\n\
        Resolving deltas:   0% (0/2226)\r\
        Resolving deltas: 100% (2226/2226), done.\n";

    #[test]
    fn separa_por_cr_e_por_lf() {
        let mut splitter = Splitter::default();
        let updates = splitter.push(REAL.as_bytes());

        // Onze atualizações no trecho; um `lines()` teria devolvido quatro.
        assert_eq!(updates.len(), 11, "{updates:#?}");
        assert_eq!(updates[0], "Cloning into 'progress-probe'...");
        assert!(updates[2].starts_with("remote: Counting objects:   0%"));
    }

    #[test]
    fn atualizacao_partida_entre_chunks_espera_o_resto() {
        let mut splitter = Splitter::default();

        // Leitura de socket corta onde quiser, inclusive no meio do número.
        assert!(splitter.push(b"Receiving objects:  34% (16").is_empty());
        let updates = splitter.push(b"766/49311)\rResolving");
        assert_eq!(updates, ["Receiving objects:  34% (16766/49311)"]);

        // O que ficou no buffer só sai no fim.
        assert_eq!(splitter.flush().as_deref(), Some("Resolving"));
        assert_eq!(splitter.flush(), None);
    }

    #[test]
    fn le_o_trecho_real_inteiro() {
        let mut splitter = Splitter::default();
        let events: Vec<_> = splitter
            .push(REAL.as_bytes())
            .iter()
            .map(|update| parse(update))
            .collect();

        assert_eq!(
            events[1],
            ProgressEvent::Enumerating {
                objects: 3986,
                done: true
            }
        );
        assert_eq!(
            events[2],
            ProgressEvent::Counting(Counted {
                percent: 0,
                current: 1,
                total: 752,
                done: false
            })
        );
        assert_eq!(
            events[3],
            ProgressEvent::Counting(Counted {
                percent: 100,
                current: 752,
                total: 752,
                done: true
            })
        );
        assert!(matches!(events[4], ProgressEvent::Compressing(_)));
        assert!(matches!(events[9], ProgressEvent::ResolvingDeltas(_)));

        // `Cloning into` e `remote: Total …` não são progresso: viram log.
        assert!(matches!(events[0], ProgressEvent::Other(_)));
        assert!(matches!(events[7], ProgressEvent::Other(_)));
    }

    #[test]
    fn receiving_traz_objetos_bytes_e_throughput() {
        // Linha real de `git clone --progress https://github.com/tokio-rs/tokio.git`.
        let event = parse("Receiving objects:  34% (16766/49311), 7.44 MiB | 14.69 MiB/s");

        assert_eq!(
            event,
            ProgressEvent::Receiving(Receiving {
                percent: 34,
                objects: 16766,
                total: 49311,
                bytes: Some(7_801_405),
                bytes_per_s: Some(15_403_581),
                done: false,
            })
        );
        assert_eq!(event.fraction(), Some(0.34));
        assert_eq!(
            event.detail().as_deref(),
            Some("16766/49311 · 7.44 MiB · 14.69 MiB/s")
        );
    }

    #[test]
    fn receiving_sem_throughput_ainda_e_progresso() {
        // No começo o git ainda não tem o que medir.
        let event = parse("Receiving objects:   0% (1/49311)");

        assert_eq!(
            event,
            ProgressEvent::Receiving(Receiving {
                percent: 0,
                objects: 1,
                total: 49311,
                bytes: None,
                bytes_per_s: None,
                done: false,
            })
        );
    }

    #[test]
    fn enumerating_nao_finge_ter_fracao() {
        let event = parse("remote: Enumerating objects: 3986, done.");

        assert_eq!(event.fraction(), None, "não há total para dividir");
        assert_eq!(event.detail().as_deref(), Some("3986 objetos"));
    }

    #[test]
    fn checkout_tambem_e_fase() {
        assert_eq!(
            parse("Updating files:  45% (123/271)"),
            ProgressEvent::CheckingOut(Counted {
                percent: 45,
                current: 123,
                total: 271,
                done: false
            })
        );
    }

    #[test]
    fn linha_desconhecida_vira_log_e_nao_derruba() {
        assert_eq!(
            parse("warning: redirecting to https://example.com/repo.git/"),
            ProgressEvent::Other(
                "warning: redirecting to https://example.com/repo.git/".to_owned()
            )
        );
        // Prefixo conhecido com números impossíveis também não pode entrar em pânico.
        assert!(matches!(
            parse("Counting objects: x% (a/b)"),
            ProgressEvent::Other(_)
        ));
    }

    #[test]
    fn tamanhos_vao_e_voltam() {
        assert_eq!(parse_size("1.11 MiB"), Some(1_163_919));
        assert_eq!(parse_size("512 B"), Some(512));
        assert_eq!(parse_size("nada"), None);
        assert_eq!(human_bytes(1 << 20), "1.00 MiB");
        assert_eq!(human_bytes(512), "512 B");
    }
}
