use std::{
    fs::{self},
    path::{Path, PathBuf},
};

use assert_fs::TempDir;
use git2::{Repository, Signature};

pub fn create_temporary_repo_with_committed_file<P: AsRef<Path>>(
    temp_dir: &TempDir,
    commit_config_toml_path: P,
) -> (PathBuf, PathBuf) {
    let repo_path = temp_dir.path().join("test-repo");
    let underlying_repo = Repository::init(&repo_path).unwrap();
    let cargo_toml_path = repo_path.join("Cargo.toml");
    let repo_cargo_toml_content = fs::read_to_string(commit_config_toml_path).unwrap();
    let () = fs::write(&cargo_toml_path, repo_cargo_toml_content).unwrap();

    // Create a git repo on-disk and add Cargo.toml to it, then commit the change
    let tree_id = {
        let mut index = underlying_repo.index().unwrap();
        let _ = index.add_path(&PathBuf::from("Cargo.toml"));
        index.write().unwrap();
        index.write_tree().unwrap()
    };
    let tree = underlying_repo.find_tree(tree_id).unwrap();
    let author = Signature::now("Test Committer", "test@example.com").unwrap();
    underlying_repo
        .commit(
            Some("HEAD"),
            &author,
            &author,
            "🌱 initial commit",
            &tree,
            &[],
        )
        .unwrap();

    (repo_path, cargo_toml_path)
}
