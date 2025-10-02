use std::path::PathBuf;

use assert_fs::{
    prelude::{FileWriteStr, PathChild},
    TempDir,
};

use crate::domain::cargo_toml::{CargoDependencyValue, DetailedCargoDependency};

use super::File;

fn get_temporary_cargo_toml_path(temp_dir: &TempDir) -> PathBuf {
    let cargo_toml_content = r#"[package]
name = "some-example-crate"
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2021"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.74"
description = "An example Rust app"

[dependencies]
ahash = "0.8.11"
clap = { version = "4.5.23", features = ["derive"] }
clap-verbosity-flag = "3.0.1"
config = "0.14.1"
env_logger = "0.11.5"
git2 = "0.19.0"
log = "0.4.22"
serde = { version = "1.0.215", features = ["derive"] }

[dependencies.sqlx]
version = "0.8.2"
default-features = false
features = ["any", "chrono", "macros", "migrate", "postgres", "runtime-tokio-rustls", "uuid"]

[dev-dependencies]
assert_fs = "1.1.2"
trycmd = "0.15.8"
"#;
    let _ = temp_dir.child("Cargo.toml").write_str(cargo_toml_content);
    temp_dir.join("Cargo.toml")
}

#[test]
fn new_successfully_parses_valid_cargo_toml_dependencies() {
    // arrange
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let temporary_cargo_toml_path = get_temporary_cargo_toml_path(&temp_dir);

    // act
    let File { dependencies, .. } = File::new(temporary_cargo_toml_path.to_str().unwrap()).unwrap();

    // assert
    assert!(dependencies.is_some());
    let Some(dependencies_value) = dependencies else {
        panic!("dependencies should be some");
    };
    assert_eq!(dependencies_value.len(), 9);
    assert_eq!(
        dependencies_value.get("ahash"),
        Some(CargoDependencyValue::Simple(String::from("0.8.11"))).as_ref()
    );
    assert_eq!(
        dependencies_value.get("serde"),
        Some(CargoDependencyValue::Detailed(DetailedCargoDependency {
            version: String::from("1.0.215"),
            package: None
        }))
        .as_ref()
    );
    assert_eq!(
        dependencies_value.get("sqlx"),
        Some(CargoDependencyValue::Detailed(DetailedCargoDependency {
            version: String::from("0.8.2"),
            package: None
        }))
        .as_ref()
    );
    insta::assert_json_snapshot!(dependencies_value);
}

#[test]
fn new_handles_missing_cargo_toml() {
    // arrange
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let temporary_cargo_toml_path = temp_dir.join("Cargoooo.toml");

    // act
    let outcome = File::new(temporary_cargo_toml_path.to_str().unwrap()).unwrap_err();

    // assert
    assert_eq!(
        format!("{outcome}"),
        format!(
            "Error opening Cargo.toml file: `{}`",
            temporary_cargo_toml_path.display()
        )
    );
    let mut chain = outcome.chain();
    assert_eq!(
        chain.next().map(|val| format!("{val}")),
        Some(format!(
            "Error opening Cargo.toml file: `{}`",
            temporary_cargo_toml_path.display()
        ))
    );
    assert_eq!(
        chain.next().map(|val| format!("{val}")),
        Some(format!(
            r#"configuration file "{}" not found"#,
            temporary_cargo_toml_path.display()
        ))
    );
    assert!(chain.next().is_none());
}

#[test]
fn new_accepts_missing_dependencies_in_cargo_toml() {
    // arrange
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let cargo_toml_content = r#"[package]
name = "some-example-crate"
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2021"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.74"
description = "An example Rust app"

[dev-dependencies]
assert_fs = "1.1.2"
trycmd = "0.15.8"
"#;
    let _ = temp_dir.child("Cargo.toml").write_str(cargo_toml_content);
    let temporary_cargo_toml_path = temp_dir.join("Cargo.toml");

    // act
    let outcome = File::new(temporary_cargo_toml_path.to_str().unwrap());

    // assert
    assert!(outcome.is_ok());
}

#[test]
fn print_changes_versus_previous_version_advises_when_there_are_no_changes() {
    let temp_dir = assert_fs::TempDir::new().unwrap();
    let temporary_cargo_toml_path = get_temporary_cargo_toml_path(&temp_dir);
    let cargo_toml_file = File::new(temporary_cargo_toml_path.to_str().unwrap()).unwrap();

    // act
    let output = cargo_toml_file
        .print_changes_versus_previous_version(&cargo_toml_file)
        .unwrap();

    // assert
    assert_eq!(output, String::from("🧹 No changes detected.\n"));
}

#[test]
fn print_dependency_changes_return_changes() {
    // arrange
    let updated_cargo_toml_content = r#"[package]
name = "some-example-crate"
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2021"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.74"
description = "An example Rust app"

[dependencies]
ahash = "0.8.11"
clap = { version = "4.5.23", features = ["derive"] }
clap-verbosity-flag = "3.0.1"
config = "0.14.1"
env_logger = "0.11.5"
git2 = "0.19.0"
log = "0.4.22"
serde = { version = "1.0.215", features = ["derive"] }

[dependencies.sqlx]
version = "0.8.2"
default-features = false
features = ["any", "chrono", "macros", "migrate", "postgres", "runtime-tokio-rustls", "uuid"]

[dev-dependencies]
assert_fs = "1.1.2"
trycmd = "0.15.8"
"#;
    let earlier_cargo_toml_content = r#"[package]
name = "some-example-crate"
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2021"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.74"
description = "An example Rust app"

[dependencies]
ahash = "0.8.10"
clap = { version = "4.5.23", features = ["derive"] }
clap-verbosity-flag = "3.0.1"
config = "0.14.1"
env_logger = "0.11.5"
git2 = "0.19.0"
log = "0.4.22"
serde = { version = "1.0.210", features = ["derive"] }

[dependencies.sqlx]
version = "0.8.2"
default-features = false
features = ["any", "chrono", "macros", "migrate", "postgres", "runtime-tokio-rustls", "uuid"]

[dev-dependencies]
assert_fs = "1.1.2"
trycmd = "0.15.8"
"#;

    let updated_cargo_toml = File::new_from_str(updated_cargo_toml_content).unwrap();
    let earlier_cargo_toml = File::new_from_str(earlier_cargo_toml_content).unwrap();
    // act
    let output = updated_cargo_toml
        .print_changes_versus_previous_version(&earlier_cargo_toml)
        .unwrap();

    // assert
    assert_eq!(
        output,
        String::from(
            "📦 bump ahash from 0.8.10 to 0.8.11
🔧 bump serde from 1.0.210 to 1.0.215
"
        )
    );
}

#[test]
fn print_dependency_changes_displays_version_drop() {
    // arrange
    let updated_cargo_toml_content = r#"[package]
name = "some-example-crate"
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2021"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.74"
description = "An example Rust app"

[dependencies]
ahash = "0.8.11"
clap = { version = "4.5.23", features = ["derive"] }
clap-verbosity-flag = "3.0.1"
config = "0.14.1"
env_logger = "0.11.5"
git2 = "0.19.0"
log = "0.4.22"
serde = { version = "1.0.210", features = ["derive"] }

[dependencies.sqlx]
version = "0.8.2"
default-features = false
features = ["any", "chrono", "macros", "migrate", "postgres", "runtime-tokio-rustls", "uuid"]

[dev-dependencies]
assert_fs = "1.1.2"
trycmd = "0.15.8"
"#;
    let earlier_cargo_toml_content = r#"[package]
name = "some-example-crate"
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2021"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.74"
description = "An example Rust app"

[dependencies]
ahash = "0.8.10"
clap = { version = "4.5.23", features = ["derive"] }
clap-verbosity-flag = "3.0.1"
config = "0.14.1"
env_logger = "0.11.5"
git2 = "0.19.0"
log = "0.4.22"
serde = { version = "1.0.215", features = ["derive"] }

[dependencies.sqlx]
version = "0.8.2"
default-features = false
features = ["any", "chrono", "macros", "migrate", "postgres", "runtime-tokio-rustls", "uuid"]

[dev-dependencies]
assert_fs = "1.1.2"
trycmd = "0.15.8"
"#;

    let updated_cargo_toml = File::new_from_str(updated_cargo_toml_content).unwrap();
    let earlier_cargo_toml = File::new_from_str(earlier_cargo_toml_content).unwrap();

    // act
    let output = updated_cargo_toml
        .print_changes_versus_previous_version(&earlier_cargo_toml)
        .unwrap();

    // assert
    assert_eq!(
        output,
        String::from(
            "📦 bump ahash from 0.8.10 to 0.8.11\n🔧 drop serde from 1.0.215 to 1.0.210\n"
        )
    );
}

#[test]
fn print_dependency_changes_displays_unclear_changes() {
    // arrange
    let updated_cargo_toml_content = r#"[package]
name = "some-example-crate"
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2021"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.74"
description = "An example Rust app"

[dependencies]
ahash = "0.8.11"
clap = { version = "4.5.23", features = ["derive"] }
clap-verbosity-flag = "3.0.1"
config = "0.14.1"
env_logger = "0.11.5"
git2 = "0.19.0"
log = "0.4.22"
serde = { version = "1", features = ["derive"] }

[dependencies.sqlx]
version = "0.8.2"
default-features = false
features = ["any", "chrono", "macros", "migrate", "postgres", "runtime-tokio-rustls", "uuid"]

[dev-dependencies]
assert_fs = "1.1.2"
trycmd = "0.15.8"
"#;
    let earlier_cargo_toml_content = r#"[package]
name = "some-example-crate"
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2021"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.74"
description = "An example Rust app"

[dependencies]
ahash = "0.8.10"
clap = { version = "4.5.23", features = ["derive"] }
clap-verbosity-flag = "3.0.1"
config = "0.14.1"
env_logger = "0.11.5"
git2 = "0.19.0"
log = "0.4.22"
serde = { version = "1.0.215", features = ["derive"] }

[dependencies.sqlx]
version = "0.8.2"
default-features = false
features = ["any", "chrono", "macros", "migrate", "postgres", "runtime-tokio-rustls", "uuid"]

[dev-dependencies]
assert_fs = "1.1.2"
trycmd = "0.15.8"
"#;

    let updated_cargo_toml = File::new_from_str(updated_cargo_toml_content).unwrap();
    let earlier_cargo_toml = File::new_from_str(earlier_cargo_toml_content).unwrap();

    // act
    let output = updated_cargo_toml
        .print_changes_versus_previous_version(&earlier_cargo_toml)
        .unwrap();

    // assert
    assert_eq!(
        output,
        String::from("📦 bump ahash from 0.8.10 to 0.8.11\n🤷 drop serde from 1.0.215 to 1\n")
    );
}

#[test]
fn print_dependency_displays_additions_and_removals() {
    // arrange
    let updated_cargo_toml_content = r#"[package]
name = "some-example-crate"
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2021"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.74"
description = "An example Rust app"

[dependencies]
ahash = "0.8.11"
clap = { version = "4.5.23", features = ["derive"] }
clap-verbosity-flag = "3.0.1"
config = "0.14.1"
env_logger = "0.11.5"
git2 = "0.19.0"
log = "0.4.22"
serde = { version = "1", features = ["derive"] }

[dependencies.sqlx]
version = "0.8.2"
default-features = false
features = ["any", "chrono", "macros", "migrate", "postgres", "runtime-tokio-rustls", "uuid"]

[dev-dependencies]
assert_fs = "1.1.2"
float-cmp = "0.10"
proptest = "1.6.0"
trycmd = "0.14"

[build-dependencies]
anyhow = "1.0.95"
fs_extra = "1.3.0"
glob = "0.3.1"
"#;
    let earlier_cargo_toml_content = r#"[package]
name = "some-example-crate"
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2021"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.74"
description = "An example Rust app"

[dependencies]
ahash = "0.8.10"
clap = { version = "4.5.23", features = ["derive"] }
clap-verbosity-flag = "3.0.1"
config = "0.14.1"
env_logger = "0.11.5"
git2 = "0.19.0"
log = "0.4.22"
image = "0.25.5"

[dependencies.sqlx]
version = "0.8.2"
default-features = false
features = ["any", "chrono", "macros", "migrate", "postgres", "runtime-tokio-rustls", "uuid"]

[dev-dependencies]
assert_fs = "0"
float-cmp = "0.10.0"
trycmd = "0.15.8"
wiremock = "0.6.2"
"#;

    let updated_cargo_toml = File::new_from_str(updated_cargo_toml_content).unwrap();
    let earlier_cargo_toml = File::new_from_str(earlier_cargo_toml_content).unwrap();

    // act
    let output = updated_cargo_toml
        .print_changes_versus_previous_version(&earlier_cargo_toml)
        .unwrap();

    // assert
    assert_eq!(
        output,
        String::from(
            "📦 bump ahash from 0.8.10 to 0.8.11\n✨ add serde 1\n🗑\u{fe0f} remove image 0.25.5\n\
                    ❗ bump assert_fs (🖥\u{fe0f} dev-dependencies) from 0 to 1.1.2\n\
                    ✨ add proptest (🖥\u{fe0f} dev-dependencies) 1.6.0\n\
                    ❗ drop trycmd (🖥\u{fe0f} dev-dependencies) from 0.15.8 to 0.14\n\
                    🗑\u{fe0f} remove wiremock (🖥\u{fe0f} dev-dependencies) 0.6.2\n\
                    ✨ add anyhow (🧱 build-dependencies) 1.0.95\n\
                    ✨ add fs_extra (🧱 build-dependencies) 1.3.0\n\
                    ✨ add glob (🧱 build-dependencies) 0.3.1\n"
        )
    );
}

#[test]
fn print_dependency_handles_build_dependency_removal() {
    // arrange
    let updated_cargo_toml_content = r#"[package]
name = "some-example-crate"
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2021"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.74"
description = "An example Rust app"

[dependencies]
ahash = "0.8.11"
clap = { version = "4.5.23", features = ["derive"] }
clap-verbosity-flag = "3.0.1"
config = "0.14.1"
env_logger = "0.11.5"
git2 = "0.19.0"
log = "0.4.22"
serde = { version = "1", features = ["derive"] }

[dev-dependencies]
assert_fs = "1.1.2"
float-cmp = "0.10"
proptest = "1.6.0"
trycmd = "0.14"

"#;
    let earlier_cargo_toml_content = r#"[package]
name = "some-example-crate"
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2021"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.74"
description = "An example Rust app"

[dependencies]
ahash = "0.8.11"
clap = { version = "4.5.23", features = ["derive"] }
clap-verbosity-flag = "3.0.1"
config = "0.14.1"
env_logger = "0.11.5"
git2 = "0.19.0"
log = "0.4.22"
serde = { version = "1", features = ["derive"] }

[dev-dependencies]
assert_fs = "1.1.2"
float-cmp = "0.10"
proptest = "1.6.0"
trycmd = "0.14"

[build-dependencies]
anyhow = "1.0.95"
fs_extra = "1.3.0"
glob = "0.3.1"
"#;

    let updated_cargo_toml = File::new_from_str(updated_cargo_toml_content).unwrap();
    let earlier_cargo_toml = File::new_from_str(earlier_cargo_toml_content).unwrap();

    // act
    let output = updated_cargo_toml
        .print_changes_versus_previous_version(&earlier_cargo_toml)
        .unwrap();

    // assert
    assert_eq!(
        output,
        String::from(
            "🗑\u{fe0f} remove anyhow (🧱 build-dependencies) 1.0.95\n\
                🗑\u{fe0f} remove fs_extra (🧱 build-dependencies) 1.3.0\n\
                🗑\u{fe0f} remove glob (🧱 build-dependencies) 0.3.1\n"
        )
    );
}

#[test]
fn print_dependency_handles_workspace_dependency_changes() {
    // arrange
    let updated_cargo_toml_content = r#"[workspace]
resolver = "2"
members = [
  "crates/number-one",
  "crates/data"
]

[workspace.package]
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2021"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.74"
description = "An example Rust app"

[workspace.dependencies]
ahash = "0.8.11"
serde = { version = "1", features = ["derive"] }
"#;
    let earlier_cargo_toml_content = r#"[workspace]
resolver = "2"
members = [
  "crates/number-one",
  "crates/data"
]

[workspace.package]
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2021"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.74"
description = "An example Rust app"

[workspace.dependencies]
ahash = "0.7"
serde = { version = "0", features = ["derive"] }
"#;

    let updated_cargo_toml = File::new_from_str(updated_cargo_toml_content).unwrap();
    let earlier_cargo_toml = File::new_from_str(earlier_cargo_toml_content).unwrap();

    // act
    let output = updated_cargo_toml
        .print_changes_versus_previous_version(&earlier_cargo_toml)
        .unwrap();

    // assert
    assert_eq!(
        output,
        String::from(
            "❗ bump ahash (🗄\u{fe0f} workspace-dependencies) from 0.7 to 0.8.11\n\
                ❗ bump serde (🗄\u{fe0f} workspace-dependencies) from 0 to 1\n"
        )
    );
}

#[test]
fn new_from_buffer_creates_expected_config() {
    // arrange
    let buffer: [u8; 713] = [
        91, 112, 97, 99, 107, 97, 103, 101, 93, 10, 110, 97, 109, 101, 32, 61, 32, 34, 115, 111,
        109, 101, 45, 101, 120, 97, 109, 112, 108, 101, 45, 99, 114, 97, 116, 101, 34, 10, 118,
        101, 114, 115, 105, 111, 110, 32, 61, 32, 34, 48, 46, 49, 46, 48, 34, 10, 97, 117, 116,
        104, 111, 114, 115, 32, 61, 32, 91, 34, 82, 117, 115, 116, 32, 67, 111, 100, 101, 114, 32,
        60, 110, 97, 109, 101, 64, 101, 120, 97, 109, 112, 108, 101, 46, 99, 111, 109, 62, 34, 93,
        10, 101, 100, 105, 116, 105, 111, 110, 32, 61, 32, 34, 50, 48, 50, 49, 34, 10, 108, 105,
        99, 101, 110, 115, 101, 32, 61, 32, 34, 66, 83, 68, 45, 51, 45, 67, 108, 97, 117, 115, 101,
        34, 10, 114, 101, 112, 111, 115, 105, 116, 111, 114, 121, 32, 61, 32, 34, 104, 116, 116,
        112, 115, 58, 47, 47, 103, 105, 116, 104, 117, 98, 46, 99, 111, 109, 47, 101, 120, 97, 109,
        112, 108, 101, 47, 101, 120, 97, 109, 112, 108, 101, 45, 114, 101, 112, 111, 34, 10, 114,
        117, 115, 116, 45, 118, 101, 114, 115, 105, 111, 110, 32, 61, 32, 34, 49, 46, 55, 52, 34,
        10, 100, 101, 115, 99, 114, 105, 112, 116, 105, 111, 110, 32, 61, 32, 34, 65, 110, 32, 101,
        120, 97, 109, 112, 108, 101, 32, 82, 117, 115, 116, 32, 97, 112, 112, 34, 10, 10, 91, 100,
        101, 112, 101, 110, 100, 101, 110, 99, 105, 101, 115, 93, 10, 97, 104, 97, 115, 104, 32,
        61, 32, 34, 48, 46, 56, 46, 49, 49, 34, 10, 99, 108, 97, 112, 32, 61, 32, 123, 32, 118,
        101, 114, 115, 105, 111, 110, 32, 61, 32, 34, 52, 46, 53, 46, 50, 51, 34, 44, 32, 102, 101,
        97, 116, 117, 114, 101, 115, 32, 61, 32, 91, 34, 100, 101, 114, 105, 118, 101, 34, 93, 32,
        125, 10, 99, 108, 97, 112, 45, 118, 101, 114, 98, 111, 115, 105, 116, 121, 45, 102, 108,
        97, 103, 32, 61, 32, 34, 51, 46, 48, 46, 49, 34, 10, 99, 111, 110, 102, 105, 103, 32, 61,
        32, 34, 48, 46, 49, 52, 46, 49, 34, 10, 101, 110, 118, 95, 108, 111, 103, 103, 101, 114,
        32, 61, 32, 34, 48, 46, 49, 49, 46, 53, 34, 10, 103, 105, 116, 50, 32, 61, 32, 34, 48, 46,
        49, 57, 46, 48, 34, 10, 108, 111, 103, 32, 61, 32, 34, 48, 46, 52, 46, 50, 50, 34, 10, 115,
        101, 114, 100, 101, 32, 61, 32, 123, 32, 118, 101, 114, 115, 105, 111, 110, 32, 61, 32, 34,
        49, 46, 48, 46, 50, 49, 53, 34, 44, 32, 102, 101, 97, 116, 117, 114, 101, 115, 32, 61, 32,
        91, 34, 100, 101, 114, 105, 118, 101, 34, 93, 32, 125, 10, 10, 91, 100, 101, 112, 101, 110,
        100, 101, 110, 99, 105, 101, 115, 46, 115, 113, 108, 120, 93, 10, 118, 101, 114, 115, 105,
        111, 110, 32, 61, 32, 34, 48, 46, 56, 46, 50, 34, 10, 100, 101, 102, 97, 117, 108, 116, 45,
        102, 101, 97, 116, 117, 114, 101, 115, 32, 61, 32, 102, 97, 108, 115, 101, 10, 102, 101,
        97, 116, 117, 114, 101, 115, 32, 61, 32, 91, 34, 97, 110, 121, 34, 44, 32, 34, 99, 104,
        114, 111, 110, 111, 34, 44, 32, 34, 109, 97, 99, 114, 111, 115, 34, 44, 32, 34, 109, 105,
        103, 114, 97, 116, 101, 34, 44, 32, 34, 112, 111, 115, 116, 103, 114, 101, 115, 34, 44, 32,
        34, 114, 117, 110, 116, 105, 109, 101, 45, 116, 111, 107, 105, 111, 45, 114, 117, 115, 116,
        108, 115, 34, 44, 32, 34, 117, 117, 105, 100, 34, 93, 10, 10, 91, 100, 101, 118, 45, 100,
        101, 112, 101, 110, 100, 101, 110, 99, 105, 101, 115, 93, 10, 97, 115, 115, 101, 114, 116,
        95, 102, 115, 32, 61, 32, 34, 49, 46, 49, 46, 50, 34, 10, 116, 114, 121, 99, 109, 100, 32,
        61, 32, 34, 48, 46, 49, 53, 46, 56, 34, 10,
    ];

    // act
    let outcome = File::new_from_buffer(&buffer).unwrap();

    // assert
    insta::assert_snapshot!(format!("{outcome:?}"));
}

#[test]
fn get_changes_from_current_dependencies_emits_package_field_for_name_when_present() {
    // arrange
    let updated_cargo_toml_content = r#"[package]
name = "some-example-crate"
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2024"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.75"
description = "An example Rust app"

[dependencies]
getrandom = { version = "0.3.2", features = ["wasm_js"] }
getrandom2 = { package = "getrandom", version = "0.2.15", features = ["js"] }
"#;

    let earlier_cargo_toml_content = r#"[package]
name = "some-example-crate"
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2024"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.75"
description = "An example Rust app"

[dependencies]
getrandom = { version = "0.3", features = ["wasm_js"] }
getrandom2 = { package = "getrandom", version = "0.2.1", features = ["js"] }
"#;
    // assert
    let updated_cargo_toml = File::new_from_str(updated_cargo_toml_content).unwrap();
    let earlier_cargo_toml = File::new_from_str(earlier_cargo_toml_content).unwrap();

    // act
    let output = updated_cargo_toml
        .print_changes_versus_previous_version(&earlier_cargo_toml)
        .unwrap();

    // assert
    assert_eq!(
        output,
        String::from(
            "🤷 bump getrandom from 0.3 to 0.3.2\n📦 bump getrandom from 0.2.1 to 0.2.15\n"
        )
    );
}

#[test]
fn get_changes_from_current_dependencies_emits_package_field_for_name_when_present_and_tracks_changed_alias(
) {
    // arrange
    let updated_cargo_toml_content = r#"[package]
name = "some-example-crate"
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2024"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.75"
description = "An example Rust app"

[dependencies]
getrandom = { version = "0.3.2", features = ["wasm_js"] }
getrandom2 = { package = "getrandom", version = "0.2.15", features = ["js"] }
"#;

    let earlier_cargo_toml_content = r#"[package]
name = "some-example-crate"
version = "0.1.0"
authors = ["Rust Coder <name@example.com>"]
edition = "2024"
license = "BSD-3-Clause"
repository = "https://github.com/example/example-repo"
rust-version = "1.75"
description = "An example Rust app"

[dependencies]
getrandom_current = { version = "0.3", features = ["wasm_js"] }
getrandom_previous = { package = "getrandom", version = "0.2.1", features = ["js"] }
"#;
    // assert
    let updated_cargo_toml = File::new_from_str(updated_cargo_toml_content).unwrap();
    let earlier_cargo_toml = File::new_from_str(earlier_cargo_toml_content).unwrap();

    // act
    let output = updated_cargo_toml
        .print_changes_versus_previous_version(&earlier_cargo_toml)
        .unwrap();

    // assert
    assert_eq!(
        output,
        String::from(
            "✨ add getrandom 0.3.2\n✨ add getrandom 0.2.15\n🗑\u{fe0f} remove getrandom_current 0.3\n🗑\u{fe0f} remove getrandom 0.2.1\n"
        )
    );
}

#[test]
fn get_changes_from_current_dependencies_reports_addition_and_removal_of_git_dependencies() {
    // arrange
    let updated_cargo_toml_content = r#"[package]
name = "gpui-tryout"
version = "0.1.0"
edition = "2021"

[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "f1af2a4a58b4e48a0ce442181120859cd4df4b30" } # v0.174.4
http_client = { git = "https://github.com/zed-industries/zed", rev = "f1af2a4a58b4e48a0ce442181120859cd4df4b30" } # v0.174.4
"#;

    let earlier_cargo_toml_content = r#"[package]
name = "gpui-tryout"
version = "0.1.0"
edition = "2021"

[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", rev = "a3f070195111f8d80111cd73b8a26d7aa2228040" } # v0.171.6
reqwest_client = { git = "https://github.com/zed-industries/zed", rev = "a3f070195111f8d80111cd73b8a26d7aa2228040" } # v0.171.6
"#;
    // assert
    let updated_cargo_toml = File::new_from_str(updated_cargo_toml_content).unwrap();
    let earlier_cargo_toml = File::new_from_str(earlier_cargo_toml_content).unwrap();

    // act
    let output = updated_cargo_toml
        .print_changes_versus_previous_version(&earlier_cargo_toml)
        .unwrap();

    // assert
    assert_eq!(
        output,
        String::from("✨ add http_client 0\n🗑\u{fe0f} remove reqwest_client 0\n")
    );
}
