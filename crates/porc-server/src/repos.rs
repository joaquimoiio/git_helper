//! Registry de repositórios abertos.
//!
//! **Nenhuma rota de git aceita caminho de repositório vindo do cliente.** Depois que o
//! usuário abre um repositório, tudo passa a ser `repo_id` — um hash opaco do caminho
//! canônico. É isso que mata path traversal na origem: não há caminho na superfície para
//! atravessar. O único lugar que recebe caminho é `POST /api/v1/repos`, que passa pelo mesmo
//! confinamento do `fs/list`.
//!
//! O registry só é preenchido por ação explícita do usuário e vive em memória: matar o
//! processo esquece tudo. A lista de recentes (Passo 26) é que persiste, e persiste caminho —
//! reabrir é sempre uma ação nova.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::RwLock,
};

use sha2::{Digest, Sha256};

/// 16 bytes em hex. Opaco e curto o bastante para caber numa URL sem incomodar; longo o
/// bastante para não haver colisão entre os repositórios de uma máquina.
const ID_BYTES: usize = 16;

/// Derivado só do caminho canônico, e não de um contador nem de um segredo do boot: reabrir o
/// mesmo repositório tem que dar o mesmo `repo_id`, senão o cache do cliente e a lista de
/// recentes ficariam apontando para ids mortos a cada boot.
pub fn repo_id(path: &Path) -> String {
    let digest = Sha256::digest(path.as_os_str().as_encoded_bytes());

    digest[..ID_BYTES]
        .iter()
        .fold(String::with_capacity(ID_BYTES * 2), |mut hex, byte| {
            use std::fmt::Write;
            write!(hex, "{byte:02x}").expect("String nunca falha em write!");
            hex
        })
}

#[derive(Default)]
pub struct Registry {
    open: RwLock<HashMap<String, PathBuf>>,
}

impl Registry {
    /// Registra (ou reafirma) um repositório e devolve o id dele.
    pub fn register(&self, path: &Path) -> String {
        let id = repo_id(path);

        // `expect` num `RwLock` envenenado: se um handler entrou em pânico segurando este
        // lock, o registry não é mais confiável e continuar seria pior do que derrubar.
        self.open
            .write()
            .expect("registry envenenado")
            .insert(id.clone(), path.to_path_buf());

        id
    }

    /// Caminho de um repositório aberto. `None` significa "id desconhecido neste boot".
    pub fn path_of(&self, id: &str) -> Option<PathBuf> {
        self.open
            .read()
            .expect("registry envenenado")
            .get(id)
            .cloned()
    }

    pub fn list(&self) -> Vec<(String, PathBuf)> {
        let open = self.open.read().expect("registry envenenado");

        let mut repos: Vec<_> = open
            .iter()
            .map(|(id, path)| (id.clone(), path.clone()))
            .collect();
        repos.sort_by(|a, b| a.1.cmp(&b.1));

        repos
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn id_e_estavel_e_distingue_caminhos() {
        let a = Path::new("/Users/joaquim/Git/porcelain");
        let b = Path::new("/Users/joaquim/Git/outro");

        assert_eq!(repo_id(a), repo_id(a));
        assert_ne!(repo_id(a), repo_id(b));
        assert_eq!(repo_id(a).len(), ID_BYTES * 2);
    }

    #[test]
    fn registry_devolve_o_caminho_do_id_registrado() {
        let registry = Registry::default();
        let path = Path::new("/Users/joaquim/Git/porcelain");

        let id = registry.register(path);
        assert_eq!(registry.path_of(&id).as_deref(), Some(path));
        assert_eq!(registry.path_of("nao-existe"), None);
        assert_eq!(registry.list().len(), 1);
    }
}
