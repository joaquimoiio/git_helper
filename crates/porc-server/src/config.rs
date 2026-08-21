//! `~/.config/porcelain/config.toml`.
//!
//! Config **nunca derruba o boot**. Arquivo ausente, ilegível ou com chave errada vira um
//! `warn` e os padrões: um cliente git que se recusa a abrir porque o TOML tem uma vírgula
//! sobrando é pior do que um que abre na home.
//!
//! As chaves são `snake_case`, ao contrário do `camelCase` da API: quem escreve este arquivo é
//! uma pessoa num editor de texto, não o frontend.

use std::path::{Path, PathBuf};

use serde::Deserialize;

const CONFIG_FILE: &str = "config.toml";

/// Três níveis abaixo da raiz cobrem `~/Git/projeto`, `~/Git/cliente/projeto` e
/// `~/code/org/time/projeto` sem varrer a home inteira.
const DEFAULT_SCAN_DEPTH: usize = 3;

/// Teto de achados por varredura. Passando disso a lista deixou de ser um atalho e virou um
/// `find` — quem tem mais repositórios que isto usa o navegador de pastas.
const DEFAULT_SCAN_LIMIT: usize = 200;

/// O arquivo, como está escrito no disco. Tudo opcional.
#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    /// Raiz do navegador de pastas e da varredura. `~` é expandido.
    root: Option<String>,
    scan_depth: Option<usize>,
    scan_limit: Option<usize>,
}

/// A config já resolvida, que é o que o servidor usa.
#[derive(Debug, Clone)]
pub struct Settings {
    /// Canônica. É o confinamento de `fs/list` e de `POST /repos`.
    pub root: PathBuf,
    pub scan_depth: usize,
    pub scan_limit: usize,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            root: default_root(),
            scan_depth: DEFAULT_SCAN_DEPTH,
            scan_limit: DEFAULT_SCAN_LIMIT,
        }
    }
}

impl Settings {
    /// Lê a config do caminho padrão do usuário.
    pub fn load() -> Self {
        let Some(path) = Self::default_path() else {
            tracing::warn!("sem diretório de config do usuário, usando os padrões");
            return Self::default();
        };

        Self::load_from(&path)
    }

    pub fn default_path() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "", "porcelain")
            .map(|dirs| dirs.config_dir().join(CONFIG_FILE))
    }

    pub fn load_from(path: &Path) -> Self {
        let raw = match std::fs::read_to_string(path) {
            Ok(raw) => raw,
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {
                tracing::debug!(path = %path.display(), "sem config, usando os padrões");
                return Self::default();
            }
            Err(err) => {
                tracing::warn!(%err, path = %path.display(), "config ilegível, usando os padrões");
                return Self::default();
            }
        };

        match toml::from_str::<File>(&raw) {
            Ok(file) => Self::from_file(file),
            Err(err) => {
                // A mensagem do `toml` aponta linha e coluna, então vale o log inteiro: é a
                // única forma de o usuário achar o erro no arquivo dele.
                tracing::warn!(%err, path = %path.display(), "config inválida, usando os padrões");
                Self::default()
            }
        }
    }

    fn from_file(file: File) -> Self {
        let defaults = Self::default();

        let root = file
            .root
            .map(|root| expand_tilde(&root))
            .map(|root| match root.canonicalize() {
                Ok(canonical) if canonical.is_dir() => canonical,
                _ => {
                    tracing::warn!(root = %root.display(), "raiz da config não existe, usando a home");
                    default_root()
                }
            })
            .unwrap_or(defaults.root);

        Self {
            root,
            // `max(1)`: profundidade zero deixaria a varredura sem nada para achar, e config
            // que se desliga sozinha em silêncio é pior do que config ignorada.
            scan_depth: file.scan_depth.unwrap_or(defaults.scan_depth).max(1),
            scan_limit: file.scan_limit.unwrap_or(defaults.scan_limit).max(1),
        }
    }
}

/// `~/Git` → `/Users/joaquim/Git`. Só o `~` inicial: `~outro` é o home de outro usuário, que
/// não temos como resolver sem consultar o sistema de contas.
fn expand_tilde(raw: &str) -> PathBuf {
    let Some(rest) = raw.strip_prefix('~') else {
        return PathBuf::from(raw);
    };

    let rest = rest.trim_start_matches(std::path::MAIN_SEPARATOR);
    home().map_or_else(|| PathBuf::from(raw), |home| home.join(rest))
}

fn home() -> Option<PathBuf> {
    directories::UserDirs::new().map(|dirs| dirs.home_dir().to_path_buf())
}

/// Raiz padrão: a home do usuário.
///
/// Cair para `/` quando não há home é deliberado — sem home o app ainda tem que abrir *alguma*
/// coisa, e o processo já roda como o usuário, então `/` não concede nada que ele não tivesse.
pub fn default_root() -> PathBuf {
    home()
        .and_then(|home| home.canonicalize().ok())
        .unwrap_or_else(|| PathBuf::from("/"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arquivo_ausente_devolve_os_padroes() {
        let settings = Settings::load_from(Path::new("/nao/existe/config.toml"));

        assert_eq!(settings.root, default_root());
        assert_eq!(settings.scan_depth, DEFAULT_SCAN_DEPTH);
    }

    #[test]
    fn toml_invalido_nao_derruba_nada() {
        let dir = std::env::temp_dir().join("porc-config-invalida");
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("config.toml");
        std::fs::write(&path, "root = [isto não é TOML").unwrap();

        assert_eq!(Settings::load_from(&path).scan_depth, DEFAULT_SCAN_DEPTH);
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn raiz_inexistente_cai_para_a_home() {
        let settings = Settings::from_file(File {
            root: Some("/nao/existe/em/lugar/nenhum".to_owned()),
            scan_depth: None,
            scan_limit: None,
        });

        assert_eq!(settings.root, default_root());
    }

    #[test]
    fn profundidade_zero_vira_um() {
        let settings = Settings::from_file(File {
            root: None,
            scan_depth: Some(0),
            scan_limit: Some(0),
        });

        assert_eq!(settings.scan_depth, 1);
        assert_eq!(settings.scan_limit, 1);
    }

    #[test]
    fn til_e_expandido_so_no_comeco() {
        let home = home().unwrap();

        assert_eq!(expand_tilde("~/Git"), home.join("Git"));
        assert_eq!(expand_tilde("~"), home);
        assert_eq!(expand_tilde("/tmp/~/x"), PathBuf::from("/tmp/~/x"));
    }
}
