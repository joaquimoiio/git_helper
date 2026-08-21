//! Descobrir repositórios no disco.
//!
//! Não abre repositório nenhum: responde só "isto aqui é um repositório?", que é o que o
//! navegador de pastas precisa saber para cada entrada que lista. Abrir um `git2::Repository`
//! por diretório listado custaria caro e não diria mais nada de útil.

use std::path::{Path, PathBuf};

/// `true` para worktree normal, worktree linkada, submódulo e repositório bare.
///
/// Em worktree normal `.git` é um diretório; em worktree linkada e em submódulo é um
/// *arquivo* com um `gitdir:` dentro. Por isso o teste é `exists()` e não `is_dir()`.
pub fn is_repo(path: &Path) -> bool {
    if path.join(".git").exists() {
        return true;
    }

    is_bare(path)
}

/// Bare não tem `.git`: o próprio diretório é o gitdir. Os três marcadores juntos evitam
/// falso positivo numa pasta qualquer que tenha um arquivo chamado `HEAD`.
fn is_bare(path: &Path) -> bool {
    path.join("HEAD").is_file() && path.join("objects").is_dir() && path.join("refs").is_dir()
}

/// Um repositório encontrado pela varredura.
#[derive(Debug, Clone)]
pub struct Found {
    pub name: String,
    pub path: PathBuf,
    /// Quantos níveis abaixo da raiz. Serve para a UI agrupar sem refazer a conta no caminho.
    pub depth: usize,
}

/// Varre `root` procurando repositórios, até `max_depth` níveis e no máximo `limit` achados.
///
/// Três regras seguram o custo, e as três são deliberadas:
///
/// - **não desce dentro de um repositório encontrado.** Submódulo e repo aninhado existem, mas
///   a varredura é para achar *projetos*, e descer num monorepo custaria minutos;
/// - **não segue symlink** (`file_type` do `DirEntry`, que não resolve). Sem isso a varredura
///   entra em ciclo e pode sair da raiz — e assim o confinamento não depende de checagem;
/// - **pula ocultos.** `.git`, `.cache`, `.venv`: nada disso é projeto do usuário.
pub fn scan(root: &Path, max_depth: usize, limit: usize) -> Vec<Found> {
    let mut found = Vec::new();
    // Fila em vez de recursão: profundidade vem do dado, e recursão sobre árvore de diretório
    // é a forma clássica de estourar a pilha com um symlink que ninguém previu.
    let mut queue = vec![(root.to_path_buf(), 0usize)];

    while let Some((dir, depth)) = queue.pop() {
        // `max_depth` conta níveis **abaixo** da raiz: 1 são os filhos diretos, 0 é nada.
        if found.len() >= limit || depth >= max_depth {
            continue;
        }

        let Ok(entries) = std::fs::read_dir(&dir) else {
            continue;
        };

        for entry in entries.flatten() {
            if found.len() >= limit {
                break;
            }

            let name = entry.file_name().to_string_lossy().into_owned();
            if name.starts_with('.') {
                continue;
            }

            if !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
                continue;
            }

            let path = entry.path();
            if is_repo(&path) {
                found.push(Found {
                    name,
                    path,
                    depth: depth + 1,
                });
            } else {
                queue.push((path, depth + 1));
            }
        }
    }

    found.sort_by(|a, b| {
        a.name
            .to_lowercase()
            .cmp(&b.name.to_lowercase())
            .then_with(|| a.path.cmp(&b.path))
    });

    found
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_root() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap()
            .to_path_buf()
    }

    #[test]
    fn varredura_acha_o_projeto_e_nao_desce_dentro_dele() {
        let parent = project_root().parent().unwrap().to_path_buf();

        let found = scan(&parent, 2, 100);
        assert!(found.iter().any(|repo| repo.path == project_root()));

        // Nada abaixo de um repositório encontrado entra na lista.
        assert!(!found
            .iter()
            .any(|repo| repo.path.starts_with(project_root().join("crates"))));
    }

    #[test]
    fn profundidade_zero_nao_acha_nada_e_o_limite_corta() {
        let parent = project_root().parent().unwrap().to_path_buf();

        assert!(scan(&parent, 0, 100).is_empty());
        assert!(scan(&parent, 3, 2).len() <= 2);
    }

    #[test]
    fn reconhece_o_proprio_repositorio_do_projeto() {
        // O crate vive dentro do repositório do porcelain, então o avô do manifesto é uma
        // worktree de verdade — não é preciso fabricar um repositório no teste.
        let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
        let root = manifest.parent().and_then(Path::parent).unwrap();

        assert!(is_repo(root));
        assert!(!is_repo(manifest));
    }
}
