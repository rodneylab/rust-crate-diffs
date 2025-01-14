use std::{
    cmp::Ordering,
    fmt::{self},
};

use semver::{Comparator, Op, VersionReq};

/// Rust Cargo.toml accepted dependency version formats:
/// - `1.2.3`
/// - `~1.2.3`
/// - `*`, `1.*`, `1.2.*`
/// - `>= 1.2.3`, `> 1.2.3`, `< 1.2.3`, `= 1.2.3`
/// - `>= 1.2, <1.5` (multiple version requirements for single dependency)
#[derive(Debug, PartialEq)]
pub enum Change {
    Major,
    Minor,
    Patch,
    None,
    Unknown,
}

impl fmt::Display for Change {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let emoji = match self {
            Change::Major => "❗",
            Change::Minor => "📦",
            Change::Patch => "🔧",
            Change::None => "😐",
            Change::Unknown => "🤷",
        };
        write!(f, "{emoji}")
    }
}

#[derive(Debug)]
pub struct Version {
    req: VersionReq,
}

/// Always skips the, implied, `^` operator in comparators
impl fmt::Display for Version {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        let Some((first, rest)) = self.req.comparators.split_first() else {
            return formatter.write_str("*");
        };

        Self::fmt_comparator_version(first, formatter)?;
        for comparator in rest {
            formatter.write_str(", ")?;
            Self::fmt_comparator_version(comparator, formatter)?;
        }

        Ok(())
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        // Wildcard and multiple version requirements not yet implemented - `new()` should not yet
        // let them be created
        debug_assert!(self.req.comparators.len() == 1 && other.req.comparators.len() == 1);
        let Comparator {
            op,
            major,
            minor,
            patch,
            ..
        } = self.req.comparators.first().expect("Index should be valid");
        match op {
            Op::Exact | Op::GreaterEq | Op::Greater | Op::Less | Op::LessEq => {
                unimplemented!("Range version requirement comparison is not yet implemented")
            }
            Op::Tilde => {
                unimplemented!("Tilde version requirement comparison is not yet implemented")
            }
            Op::Caret => {}
            Op::Wildcard => {
                unimplemented!("Wildcard version requirement comparison is not yet implemented")
            }
            _ => unimplemented!(
                "Unexpected version requirement. Requirement type is not yet implemented"
            ),
        }
        let Comparator {
            op: other_op,
            major: other_major,
            minor: other_minor,
            patch: other_patch,
            ..
        } = other
            .req
            .comparators
            .first()
            .expect("Index should be valid");
        match other_op {
            Op::Exact | Op::GreaterEq | Op::Greater | Op::Less | Op::LessEq => {
                unimplemented!("Range version requirement comparison is not yet implemented")
            }
            Op::Tilde => {
                unimplemented!("Tilde version requirement comparison is not yet implemented")
            }
            Op::Caret => {}
            Op::Wildcard => {
                unimplemented!("Wildcard version requirement comparison is not yet implemented")
            }
            _ => unimplemented!(
                "Unexpected version requirement. Requirement type is not yet implemented"
            ),
        }
        if minor.is_some() && other_minor.is_some() {
            if (patch.is_some() && other_patch.is_some())
                || (patch.is_none() && other_patch.is_none())
            {
                Some(
                    major
                        .cmp(other_major)
                        .then(minor.cmp(other_minor))
                        .then(patch.cmp(other_patch)),
                )
            } else if major != other_major || minor != other_minor {
                Some(major.cmp(other_major).then(minor.cmp(other_minor)))
            } else {
                None
            }
        } else if (minor.is_none()
            && other_minor.is_none()
            && patch.is_none()
            && other_patch.is_none())
            || (major != other_major
                && ((minor.is_none() && patch.is_none())
                    || (other_minor.is_none() && other_patch.is_none())))
        {
            Some(major.cmp(other_major))
        } else {
            None
        }
    }
}

impl PartialEq for Version {
    fn eq(&self, other: &Self) -> bool {
        self.req == other.req
    }
}

impl Eq for Version {}

impl Version {
    pub fn new(version_number: &str) -> Result<Self, String> {
        let req = VersionReq::parse(version_number).map_err(|error| format!("{error}"))?;

        let () = Self::error_if_comparator_operator_not_supported(&req)?;

        Ok(Self { req })
    }

    pub fn change_type(&self, other: &Self) -> Change {
        let Comparator {
            major,
            minor,
            patch,
            ..
        } = self.req.comparators.first().expect("Index should be valid");
        let Comparator {
            major: other_major,
            minor: other_minor,
            patch: other_patch,
            ..
        } = other
            .req
            .comparators
            .first()
            .expect("Index should be valid");
        debug_assert!(minor.is_some() || patch.is_none());
        debug_assert!(other_minor.is_some() || other_patch.is_none());
        if major != other_major {
            return Change::Major;
        }
        if let (Some(self_minor), Some(other_minor)) = (minor, other_minor) {
            if self_minor != other_minor {
                if *major > 0 {
                    return Change::Minor;
                }
                return Change::Major;
            }
            if let (Some(self_patch), Some(other_patch)) = (patch, other_patch) {
                if self_patch != other_patch {
                    if *major > 0 {
                        return Change::Patch;
                    }
                    if *self_minor > 0 {
                        return Change::Minor;
                    }
                    return Change::Major;
                }
                return Change::None;
            }
        }

        Change::Unknown
    }

    fn error_if_comparator_operator_not_supported(req: &VersionReq) -> Result<(), String> {
        if req.comparators.len() != 1 {
            return Err(String::from(
                "Multiple version requirement comparison is not yet implemented",
            ));
        }
        let Comparator { op, .. } = req.comparators.first().expect("Index should be valid");
        match op {
            Op::Exact | Op::GreaterEq | Op::Greater | Op::Less | Op::LessEq => Err(String::from(
                "Range version requirement comparison is not yet implemented",
            )),
            Op::Tilde => Err(String::from(
                "Tilde version requirement comparison is not yet implemented",
            )),
            Op::Caret => Ok(()),
            Op::Wildcard => Err(String::from(
                "Wildcard version requirement comparison is not yet implemented",
            )),
            _ => Err(String::from(
                "Unexpected version requirement. Requirement type is not yet implemented",
            )),
        }
    }

    fn fmt_comparator_version(
        comparator: &Comparator,
        formatter: &mut fmt::Formatter,
    ) -> fmt::Result {
        match comparator.op {
            Op::Caret => {
                let Comparator {
                    major,
                    minor,
                    patch,
                    pre,
                    ..
                } = comparator;
                write!(formatter, "{major}")?;
                if let Some(minor_value) = minor {
                    write!(formatter, ".{minor_value}")?;
                    if let Some(patch_value) = patch {
                        write!(formatter, ".{patch_value}")?;
                        if !(pre.is_empty()) {
                            write!(formatter, "-{pre}")?;
                        }
                    }
                }
                Ok(())
            }
            _ => write!(formatter, "{comparator}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use semver::{Comparator, Op, Prerelease, VersionReq};

    use super::Change;
    use crate::domain::SemverVersion;

    #[test]
    fn fmt_semver_change_displays_expected_values() {
        // act
        let result = format!(
            "Major: {}\nMinor: {}\nPatch: {}\nNone: {}\nUnknown: {}",
            Change::Major,
            Change::Minor,
            Change::Patch,
            Change::None,
            Change::Unknown
        );

        // assert
        insta::assert_snapshot!(result);
    }

    #[test]
    fn semver_version_applies_partial_order_as_expected() {
        // assert
        assert!(SemverVersion::new("1.2.3").unwrap() < SemverVersion::new("1.2.4").unwrap());
        assert!(SemverVersion::new("1.2.3").unwrap() < SemverVersion::new("1.3.2").unwrap());
        assert!(SemverVersion::new("1.2.3").unwrap() < SemverVersion::new("2.1.2").unwrap());
        assert!(SemverVersion::new("1.2").unwrap() < SemverVersion::new("1.3").unwrap());
        assert!(SemverVersion::new("1.2").unwrap() < SemverVersion::new("2.1").unwrap());
        assert!(SemverVersion::new("10").unwrap() < SemverVersion::new("200").unwrap());
        assert_eq!(
            SemverVersion::new("1.2.3")
                .unwrap()
                .partial_cmp(&SemverVersion::new("1.2").unwrap()),
            None
        );
        assert_eq!(
            SemverVersion::new("1.2.3")
                .unwrap()
                .partial_cmp(&SemverVersion::new("1").unwrap()),
            None
        );
        assert_eq!(
            SemverVersion::new("1.2")
                .unwrap()
                .partial_cmp(&SemverVersion::new("1").unwrap()),
            None
        );
    }

    #[test]
    fn semver_version_applies_partial_equal_as_expected() {
        // assert
        assert_eq!(
            SemverVersion::new("1.2.3").unwrap(),
            SemverVersion::new("1.2.3").unwrap()
        );
        assert!(SemverVersion::new("1.2.3").unwrap() != SemverVersion::new("1.2.4").unwrap());
        assert_eq!(
            SemverVersion::new("1.2").unwrap(),
            SemverVersion::new("1.2").unwrap()
        );
        assert!(SemverVersion::new("1.2").unwrap() != SemverVersion::new("1.3").unwrap());
        assert_eq!(
            SemverVersion::new("1").unwrap(),
            SemverVersion::new("1").unwrap()
        );
        assert!(SemverVersion::new("10").unwrap() != SemverVersion::new("200").unwrap());
        assert!(SemverVersion::new("1").unwrap() != SemverVersion::new("1.0").unwrap());
        assert!(SemverVersion::new("2.1").unwrap() != SemverVersion::new("2.1.0").unwrap());
    }

    #[test]
    fn semver_version_parses_valid_semver_strings() {
        // assert
        assert_eq!(
            SemverVersion::new("1.2.3").unwrap(),
            SemverVersion {
                req: VersionReq {
                    comparators: vec![Comparator {
                        op: Op::Caret,
                        major: 1,
                        minor: Some(2),
                        patch: Some(3),
                        pre: Prerelease::EMPTY,
                    }]
                }
            }
        );
        assert_eq!(
            SemverVersion::new("1.2").unwrap(),
            SemverVersion {
                req: VersionReq {
                    comparators: vec![Comparator {
                        op: Op::Caret,
                        major: 1,
                        minor: Some(2),
                        patch: None,
                        pre: Prerelease::EMPTY,
                    }]
                }
            }
        );
        assert_eq!(
            SemverVersion::new("1").unwrap(),
            SemverVersion {
                req: VersionReq {
                    comparators: vec![Comparator {
                        op: Op::Caret,
                        major: 1,
                        minor: None,
                        patch: None,
                        pre: Prerelease::EMPTY,
                    }]
                }
            }
        );
        assert_eq!(
            SemverVersion::new("0.0.1-alpha.0").unwrap(),
            SemverVersion {
                req: VersionReq {
                    comparators: vec![Comparator {
                        op: Op::Caret,
                        major: 0,
                        minor: Some(0),
                        patch: Some(1),
                        pre: Prerelease::new("alpha.0").unwrap(),
                    }]
                }
            }
        );
    }

    #[test]
    fn semver_version_catches_invalid_semver_strings() {
        // assert
        assert_eq!(
            SemverVersion::new("1..3").unwrap_err(),
            String::from("unexpected character '.' while parsing minor version number")
        );
        assert_eq!(
            SemverVersion::new("1.").unwrap_err(),
            String::from("unexpected end of input while parsing minor version number")
        );
        assert_eq!(
            SemverVersion::new("xyz").unwrap_err(),
            String::from("unexpected character after wildcard in version req")
        );
        assert_eq!(
            SemverVersion::new(".2").unwrap_err(),
            String::from("unexpected character '.' while parsing major version number")
        );
        assert_eq!(
            SemverVersion::new(">2.1.3").unwrap_err(),
            String::from("Range version requirement comparison is not yet implemented")
        );
    }

    #[test]
    fn change_type_returns_expected_values() {
        // assert
        assert_eq!(
            SemverVersion::new("1.2.3")
                .unwrap()
                .change_type(&SemverVersion::new("1.2.3").unwrap()),
            Change::None
        );
        assert_eq!(
            SemverVersion::new("1.2.3")
                .unwrap()
                .change_type(&SemverVersion::new("2.2.3").unwrap()),
            Change::Major
        );
        assert_eq!(
            SemverVersion::new("0.2.3")
                .unwrap()
                .change_type(&SemverVersion::new("0.3.3").unwrap()),
            Change::Major
        );
        assert_eq!(
            SemverVersion::new("0.0.3")
                .unwrap()
                .change_type(&SemverVersion::new("0.0.4").unwrap()),
            Change::Major
        );
        assert_eq!(
            SemverVersion::new("1.2.3")
                .unwrap()
                .change_type(&SemverVersion::new("1.3.3").unwrap()),
            Change::Minor
        );
        assert_eq!(
            SemverVersion::new("0.1.2")
                .unwrap()
                .change_type(&SemverVersion::new("0.1.3").unwrap()),
            Change::Minor
        );
        assert_eq!(
            SemverVersion::new("1.2.3")
                .unwrap()
                .change_type(&SemverVersion::new("1.2.4").unwrap()),
            Change::Patch
        );
        assert_eq!(
            SemverVersion::new("1.2.3")
                .unwrap()
                .change_type(&SemverVersion::new("1").unwrap()),
            Change::Unknown
        );
        assert_eq!(
            SemverVersion::new("1.2.3")
                .unwrap()
                .change_type(&SemverVersion::new("1.2").unwrap()),
            Change::Unknown
        );
        assert_eq!(
            SemverVersion::new("1.2")
                .unwrap()
                .change_type(&SemverVersion::new("1").unwrap()),
            Change::Unknown
        );
    }

    #[test]
    fn fmt_returns_expected_value_for_prerelease_requirement() {
        // arrange
        let version = SemverVersion {
            req: VersionReq {
                comparators: vec![Comparator {
                    op: Op::Caret,
                    major: 0,
                    minor: Some(0),
                    patch: Some(1),
                    pre: Prerelease::new("alpha.0").unwrap(),
                }],
            },
        };

        // act
        let outcome = format!("{version}");

        // assert
        assert_eq!(outcome, String::from("0.0.1-alpha.0"));
    }
}
