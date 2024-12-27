use std::{fmt, io::Write, path::Path};

use anyhow::Context;
use git2::Repository;

pub struct Repo {
    repository: Repository,
}

impl fmt::Display for Repo {
    /// Display repo
    ///
    /// Function generates a canonical path for the repo, and the code assumes that the repo path
    /// is valid
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Repo {{ Path: {}, State: {:?} }}",
            self.repository
                .path()
                .canonicalize()
                .expect("Path should be valid")
                .display(),
            self.repository.state(),
        )
    }
}

impl fmt::Debug for Repo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl Repo {
    pub fn new<P: AsRef<Path>>(local_path: P) -> anyhow::Result<Self> {
        Repository::open(&local_path)
            .map(|repository| Self { repository })
            .with_context(|| format!("Failed to open repo: `{}`", local_path.as_ref().display()))
    }

    pub fn get_committed_cargo_toml(&self, buffer: &mut Vec<u8>) -> anyhow::Result<()> {
        let main_tree = self
            .repository
            .revparse_single("HEAD^{tree}")
            .context( "Unable to access git repo branch head.  Is the project within an existing git repo?") ?
            .peel_to_tree()
            .context("Get repo default branch tree")?;
        let file_entry = main_tree
            .iter()
            .find(|val| val.name() == Some("Cargo.toml"))
            .context("No Cargo.toml found in route directory of Git branch")?;

        let file_object = file_entry
            .to_object(&self.repository)
            .context("Convert Cargo.toml file entry to object")?;
        let file_blob = file_object
            .as_blob()
            .context("Convert Cargo.toml file entry to blob")?;
        buffer
            .write_all(file_blob.content())
            .context("Copy Cargo.toml content to temporary buffer")?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::{self, fs};

    use git2::Repository;

    use crate::{domain::Repo, test_helpers::create_temporary_repo_with_committed_file};

    #[test]
    fn new_outputs_error_if_repo_does_not_exist() {
        // arrange

        // act
        let outcome = Repo::new("path-does-not-exist").unwrap_err();

        // assert
        assert_eq!(
            format!("{outcome}"),
            "Failed to open repo: `path-does-not-exist`"
        );
        let mut chain = outcome.chain();
        assert_eq!(
            chain.next().map(|val| format!("{val}")),
            Some(String::from("Failed to open repo: `path-does-not-exist`"))
        );
    }

    #[test]
    fn new_successfully_opens_existing_repo() {
        // arrange
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let repo_path = temp_dir.join("test-repo");
        let _temp_repo = Repository::init(&repo_path);

        // act
        let outcome = Repo::new(&repo_path).unwrap();

        // assert
        assert_eq!(
            format!("{outcome}"),
            format!(
                "Repo {{ Path: {}/.git, State: Clean }}",
                repo_path.canonicalize().unwrap().to_str().unwrap()
            )
        );
    }

    #[test]
    fn fmt_displays_repo() {
        // arrange
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test-repo");
        let _ = Repository::init(&repo_path);
        let repo = Repo::new(&repo_path).unwrap();

        // act
        let result = repo.to_string();

        // assert
        assert_eq!(
            result,
            format!(
                "Repo {{ Path: {}/.git, State: Clean }}",
                repo_path.canonicalize().unwrap().to_str().unwrap()
            )
        );
    }

    #[test]
    fn debug_fmt_displays_repo() {
        // arrange
        let temp_dir = assert_fs::TempDir::new().unwrap();
        let repo_path = temp_dir.path().join("test-repo");
        let _ = Repository::init(&repo_path);
        let repo = Repo::new(&repo_path).unwrap();

        // act
        let result = format!("{repo:?}");

        // assert
        assert_eq!(
            result,
            format!(
                "Repo {{ Path: {}/.git, State: Clean }}",
                repo_path.canonicalize().unwrap().to_str().unwrap()
            )
        );
    }

    #[test]
    fn get_committed_cargo_toml_retrieves_expected_file() {
        // arrange
        let temp_dir = assert_fs::TempDir::new().unwrap();

        let (repo_path, _cargo_toml_path) = create_temporary_repo_with_committed_file(
            &temp_dir,
            "src/domain/repo/test_fixtures/cargo_toml_repo.toml",
        );

        // repo object from this module for testing
        let repo = Repo::new(&repo_path).unwrap();

        // act
        let mut result: Vec<u8> = Vec::new();
        repo.get_committed_cargo_toml(&mut result).unwrap();

        // assert
        let initial_cargo_toml =
            fs::read_to_string("src/domain/repo/test_fixtures/cargo_toml_repo.toml").unwrap();
        assert_eq!(std::str::from_utf8(&result).unwrap(), initial_cargo_toml);
    }
}
