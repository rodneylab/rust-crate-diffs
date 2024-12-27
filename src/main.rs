#![warn(clippy::all, clippy::pedantic)]

mod cli;
mod domain;

#[cfg(test)]
mod test_helpers;

use std::path::Path;

use anyhow::Context;
use clap::Parser;

use crate::{
    cli::Cli,
    domain::{CargoTomlFile, Repo},
};

fn get_rust_crate_diffs<P: AsRef<Path>>(repo_path: P) -> anyhow::Result<String> {
    let repo = Repo::new(repo_path.as_ref()).with_context(|| {
        format!(
            "Failed to open repo at `{}`, check the path is correct.",
            repo_path.as_ref().display()
        )
    })?;

    let cargo_toml_path = format!("{}/Cargo.toml", repo_path.as_ref().display());
    let latest_cargo_toml_file =
        CargoTomlFile::new(&cargo_toml_path).context("Open latest Cargo.toml file")?;

    let mut original_cargo_toml_buffer: Vec<u8> = Vec::new();
    repo.get_committed_cargo_toml(&mut original_cargo_toml_buffer)
        .context("Get committed Cargo.toml file")?;
    let original_cargo_toml_file = CargoTomlFile::new_from_buffer(&original_cargo_toml_buffer)?;

    latest_cargo_toml_file.print_changes_versus_previous_version(&original_cargo_toml_file)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = &Cli::parse();
    env_logger::Builder::new()
        .filter_level(cli.verbose.log_level_filter())
        .init();

    let repo_path = &cli.repo_path;

    let output = get_rust_crate_diffs(repo_path)?;
    for line in output.lines() {
        println!("{line}");
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs::{self};

    use super::test_helpers::create_temporary_repo_with_committed_file;
    use crate::get_rust_crate_diffs;

    /// Runs snapshots again input Cargo file in `src/snapshot_inputs`.  Requires a pair of input
    /// files to exist for each test:
    ///
    ///     - `src/snapshot_inputs/<some_stem>_repo.toml`; and
    ///     - `src/snapshot_inputs/<some_stem>_local.toml`.
    #[test]
    fn get_rust_crate_diffs_returns_expected_result() {
        insta::glob!(
            "snapshot_inputs/*_repo.toml",
            |input_repo_cargo_toml_path| {
                // arrange
                let input_repo_cargo_toml_path_str: String =
                    input_repo_cargo_toml_path.to_str().unwrap().to_string();
                let tail_index = input_repo_cargo_toml_path_str.len() - 10;
                let path_stem = &input_repo_cargo_toml_path_str[..tail_index];
                let input_local_cargo_toml_path = format!("{path_stem}_local.toml",);
                assert!(fs::exists(&input_local_cargo_toml_path).is_ok());

                let temp_dir = assert_fs::TempDir::new().unwrap();
                let (repo_path, cargo_toml_path) = create_temporary_repo_with_committed_file(
                    &temp_dir,
                    input_repo_cargo_toml_path,
                );

                // make changes to the on-disk Cargo.toml
                let local_cargo_toml_content =
                    fs::read_to_string(&input_local_cargo_toml_path).unwrap();
                let () = fs::write(&cargo_toml_path, local_cargo_toml_content).unwrap();

                // act
                let result = get_rust_crate_diffs(repo_path).unwrap();

                // assert
                insta::assert_snapshot!(result);
            }
        );
    }
}
