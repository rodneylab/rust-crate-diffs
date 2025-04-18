#[cfg(test)]
mod tests;

use core::str;
use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
};

use anyhow::{anyhow, Context};
use config::Config;
use serde::Deserialize;

use super::SemverVersion;

#[derive(Debug)]
pub struct File {
    dependencies: Option<BTreeMap<String, CargoDependencyValue>>,
    build_dependencies: Option<BTreeMap<String, CargoDependencyValue>>,
    dev_dependencies: Option<BTreeMap<String, CargoDependencyValue>>,
    workspace_dependencies: Option<BTreeMap<String, CargoDependencyValue>>,
}

impl File {
    pub fn new(path: &str) -> anyhow::Result<Self> {
        let current_cargo = Config::builder()
            .add_source(config::File::with_name(path))
            .build()
            .with_context(|| format!("Error opening Cargo.toml file: `{path}`"))?;
        let CargoFile {
            dependencies,
            build_dependencies,
            dev_dependencies,
            workspace,
        } = current_cargo
            .try_deserialize::<CargoFile>()
            .with_context(|| format!("Error parsing `{path}`"))?;

        let workspace_dependencies = if let Some(workspace_val) = workspace {
            workspace_val.dependencies
        } else {
            None
        };
        log::trace!("Cargo dependencies: {dependencies:?}");
        log::trace!("Cargo build-dependencies: {build_dependencies:?}");
        log::trace!("Cargo dev-dependencies: {dev_dependencies:?}");
        log::trace!("Cargo workspace-dependencies: {workspace_dependencies:?}");

        Ok(Self {
            dependencies,
            build_dependencies,
            dev_dependencies,
            workspace_dependencies,
        })
    }

    pub fn new_from_buffer(buffer: &[u8]) -> anyhow::Result<Self> {
        let cargo_toml_str = str::from_utf8(buffer).context("Creating `CargoFile` from buffer")?;

        Self::new_from_str(cargo_toml_str)
    }

    pub fn new_from_str(toml_str: &str) -> anyhow::Result<Self> {
        let CargoFile {
            dependencies,
            build_dependencies,
            dev_dependencies,
            workspace,
        } = toml::from_str(toml_str).context("Creating `CargoFile` from str")?;
        log::trace!("Cargo: {dependencies:?}");

        let workspace_dependencies = if let Some(workspace_val) = workspace {
            workspace_val.dependencies
        } else {
            None
        };

        Ok(Self {
            dependencies,
            build_dependencies,
            dev_dependencies,
            workspace_dependencies,
        })
    }

    fn get_version(value: &CargoDependencyValue) -> anyhow::Result<SemverVersion> {
        match value {
            CargoDependencyValue::Simple(version) => SemverVersion::new(version).map_err(|error| {
                anyhow!(
                    "Unexpected semver {version} found while computing dependency changes: \
                            {error}",
                )
            }),
            CargoDependencyValue::Detailed(DetailedCargoDependency { version, .. }) => {
                SemverVersion::new(version).map_err(|_| {
                    anyhow!(
                        "Unexpected semver version `{version}` found while computing \
                            dependency changes"
                    )
                })
            }
            CargoDependencyValue::Git(GitCargoDependency { git, .. }) => {
                log::warn!(
                    "Git dependency `{git}` found, but version change detection for git \
                        dependencies is not currently supported"
                );
                SemverVersion::new("0").map_err(|_| unreachable!("Version 0 should be valid"))
            }
        }
    }

    fn get_changes_from_current_dependencies(
        current_dependencies: &BTreeMap<String, CargoDependencyValue>,
        previous_dependencies: &BTreeMap<String, CargoDependencyValue>,
        label: Option<&str>,
        previous_keys: &mut BTreeSet<String>,
        result: &mut String,
    ) -> anyhow::Result<()> {
        for (name, current_value) in current_dependencies {
            let current_version = Self::get_version(current_value)?;
            let package_name = match current_value {
                CargoDependencyValue::Simple(_) => name,
                CargoDependencyValue::Git(GitCargoDependency { package, .. })
                | CargoDependencyValue::Detailed(DetailedCargoDependency { package, .. }) => {
                    if let Some(package_value) = package {
                        package_value
                    } else {
                        name
                    }
                }
            };
            if let Some(previous_value) = previous_dependencies.get(name) {
                // Handle dependencies in previous and current (filtering for ones with changed
                // versions)
                let previous_version = Self::get_version(previous_value)?;

                // Housekeeping to make previous keys into a list of only crates removed in the
                // current Cargo.toml
                previous_keys.remove(name);

                let change_type = current_version.change_type(&previous_version);
                match current_version.partial_cmp(&previous_version) {
                    Some(Ordering::Greater) => {
                        if let Some(label_value) = label {
                            let _ =
                                writeln!(result,
                                "{change_type} bump {package_name} {label_value} from {previous_version} \
                                    to {current_version}",
                            );
                        } else {
                            let _ = writeln!(
                                result,
                                "{change_type} bump {package_name} from {previous_version} to \
                                    {current_version}",
                            );
                        }
                    }
                    Some(Ordering::Equal) => {}
                    Some(Ordering::Less) => {
                        if let Some(label_value) = label {
                            let _ = writeln!(result,
                            "{change_type} drop {package_name} {label_value} from {previous_version} to \
                                {current_version}"
                        );
                        } else {
                            let _ = writeln!(
                                result,
                                "{change_type} drop {package_name} from {previous_version} to \
                                {current_version}"
                            );
                        }
                    }
                    None => {
                        if let Some(label_value) = label {
                            let _ = writeln!(result,
                                "{change_type} change {package_name} {label_value} from {previous_version} \
                                to {current_version}\n"
                            );
                        } else {
                            let _ = writeln!(
                                result,
                                "{change_type} change {package_name} from {previous_version} to \
                                {current_version}"
                            );
                        }
                    }
                }
            } else {
                // Handle added dependencies
                if let Some(label_value) = label {
                    let _ = writeln!(
                        result,
                        "✨ add {package_name} {label_value} {current_version}"
                    );
                } else {
                    let _ = writeln!(result, "✨ add {package_name} {current_version}");
                }
            }
        }

        Ok(())
    }

    fn get_dependency_changes_versus_previous(
        current_dependencies: &BTreeMap<String, CargoDependencyValue>,
        previous_dependencies: &BTreeMap<String, CargoDependencyValue>,
        label: Option<&str>,
        result: &mut String,
    ) -> anyhow::Result<()> {
        // Update incrementally eventually leaving only previous dependencies (that are no longer
        // dependencies)
        let mut previous_keys: BTreeSet<_> = previous_dependencies.keys().cloned().collect();

        Self::get_changes_from_current_dependencies(
            current_dependencies,
            previous_dependencies,
            label,
            &mut previous_keys,
            result,
        )?;

        // Handle removed dependencies
        for name in previous_keys {
            let (version, package_name): (SemverVersion, &str) = match previous_dependencies
                .get(&name)
                .expect("Previous dependencies should include this dependency.")
            {
                CargoDependencyValue::Simple(version) => {
                    let version = SemverVersion::new(version).unwrap();
                    (version, &name)
                }
                CargoDependencyValue::Detailed(DetailedCargoDependency { package, version }) => {
                    let version = SemverVersion::new(version)
                        .expect("Previous dependencies should include this dependency.");
                    let name = if let Some(package_value) = package {
                        package_value
                    } else {
                        &name
                    };
                    (version, name)
                }
                CargoDependencyValue::Git(GitCargoDependency { git, package }) => {
                    log::warn!("Git dependency `{git}` found, but version change detection for git dependencies is not currently supported");
                    let version =
                        SemverVersion::new("0").expect("`0` should be a valid semver version");
                    let name = if let Some(package_value) = package {
                        package_value
                    } else {
                        &name
                    };
                    (version, name)
                }
            };
            if let Some(label_value) = label {
                let _ = writeln!(result, "🗑️ remove {package_name} {label_value} {version}");
            } else {
                let _ = writeln!(result, "🗑️ remove {package_name} {version}");
            }
        }

        Ok(())
    }

    fn get_optional_dependency_changes_versus_previous(
        current_dependencies: Option<&BTreeMap<String, CargoDependencyValue>>,
        previous_dependencies: Option<&BTreeMap<String, CargoDependencyValue>>,
        label: Option<&str>,
        result: &mut String,
    ) -> anyhow::Result<()> {
        match (current_dependencies, previous_dependencies) {
            (Some(current_value), Some(previous_value)) => {
                Self::get_dependency_changes_versus_previous(
                    current_value,
                    previous_value,
                    label,
                    result,
                )?;
            }
            (Some(current_value), None) => {
                let previous = BTreeMap::<String, CargoDependencyValue>::new();
                Self::get_dependency_changes_versus_previous(
                    current_value,
                    &previous,
                    label,
                    result,
                )?;
            }
            (None, Some(previous_value)) => {
                let current = BTreeMap::<String, CargoDependencyValue>::new();
                Self::get_dependency_changes_versus_previous(
                    &current,
                    previous_value,
                    label,
                    result,
                )?;
            }
            (None, None) => {}
        }
        Ok(())
    }

    pub fn print_changes_versus_previous_version(&self, previous: &Self) -> anyhow::Result<String> {
        let mut result: String = String::new();

        Self::get_optional_dependency_changes_versus_previous(
            self.dependencies.as_ref(),
            previous.dependencies.as_ref(),
            None,
            &mut result,
        )?;

        Self::get_optional_dependency_changes_versus_previous(
            self.dev_dependencies.as_ref(),
            previous.dev_dependencies.as_ref(),
            Some("(🖥️ dev-dependencies)"),
            &mut result,
        )?;

        Self::get_optional_dependency_changes_versus_previous(
            self.build_dependencies.as_ref(),
            previous.build_dependencies.as_ref(),
            Some("(🧱 build-dependencies)"),
            &mut result,
        )?;

        Self::get_optional_dependency_changes_versus_previous(
            self.workspace_dependencies.as_ref(),
            previous.workspace_dependencies.as_ref(),
            Some("(🗄️ workspace-dependencies)"),
            &mut result,
        )?;

        if result.is_empty() {
            return Ok(String::from("🧹 No changes detected.\n"));
        }

        Ok(result)
    }
}

#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct DetailedCargoDependency {
    // #[allow(dead_code, reason = "Field needed for deserialisation")]
    #[allow(dead_code)]
    version: String,
    package: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct GitCargoDependency {
    // #[allow(dead_code, reason = "Field needed for deserialisation")]
    #[allow(dead_code)]
    git: String,
    package: Option<String>,
}

#[derive(Debug, Deserialize, PartialEq)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(untagged)]
pub enum CargoDependencyValue {
    // #[allow(dead_code, reason = "Field needed for deserialisation")]
    #[allow(dead_code)]
    Simple(String),

    // #[allow(dead_code, reason = "Field needed for deserialisation")]
    #[allow(dead_code)]
    Detailed(DetailedCargoDependency),

    // #[allow(dead_code, reason = "Field needed for deserialisation")]
    #[allow(dead_code)]
    Git(GitCargoDependency),
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
pub struct CargoWorkspace {
    pub dependencies: Option<BTreeMap<String, CargoDependencyValue>>,
}

#[derive(Debug, Deserialize)]
#[cfg_attr(test, derive(serde::Serialize))]
#[serde(rename_all = "kebab-case")]
pub struct CargoFile {
    pub dependencies: Option<BTreeMap<String, CargoDependencyValue>>,
    pub build_dependencies: Option<BTreeMap<String, CargoDependencyValue>>,
    pub dev_dependencies: Option<BTreeMap<String, CargoDependencyValue>>,
    pub workspace: Option<CargoWorkspace>,
}
