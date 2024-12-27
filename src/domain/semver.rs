use std::{cmp::Ordering, fmt};

/// Rust Cargo.toml accepted dependency version formats:
/// - `1.2.3`
/// - `~1.2.3`
/// - `*`, `1.*`, `1.2.*`
/// - `>= 1.2.3`, `> 1.2.3`, `< 1.2.3`, `= 1.2.3`
/// - `>= 1.2, <1.5` (multiple version requirements for single dependency)
#[derive(Debug, PartialEq)]
pub enum SemverChange {
    Major,
    Minor,
    Patch,
    None,
    Unknown,
}

impl fmt::Display for SemverChange {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let emoji = match self {
            SemverChange::Major => "❗",
            SemverChange::Minor => "📦",
            SemverChange::Patch => "🔧",
            SemverChange::None => "😐",
            SemverChange::Unknown => "🤷",
        };
        write!(f, "{emoji}")
    }
}

#[derive(Debug)]
pub struct SemverVersion {
    major: u32,
    minor: Option<u32>,
    patch: Option<u32>,
}

impl fmt::Display for SemverVersion {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self.minor {
            Some(minor_value) => match self.patch {
                Some(patch_value) => write!(f, "{}.{minor_value}.{patch_value}", self.major),
                None => write!(f, "{}.{minor_value}", self.major),
            },
            None => write!(f, "{}", self.major),
        }
    }
}

impl PartialOrd for SemverVersion {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.minor.is_some() && other.minor.is_some() {
            if (self.patch.is_some() && other.patch.is_some())
                || (self.patch.is_none() && other.patch.is_none())
            {
                Some(
                    self.major
                        .cmp(&other.major)
                        .then(self.minor.cmp(&other.minor))
                        .then(self.patch.cmp(&other.patch)),
                )
            } else if self.major != other.major || self.minor != other.minor {
                Some(
                    self.major
                        .cmp(&other.major)
                        .then(self.minor.cmp(&other.minor)),
                )
            } else {
                None
            }
        } else if (self.minor.is_none()
            && other.minor.is_none()
            && self.patch.is_none()
            && other.patch.is_none())
            || (self.major != other.major
                && ((self.minor.is_none() && self.patch.is_none())
                    || (other.minor.is_none() && other.patch.is_none())))
        {
            Some(self.major.cmp(&other.major))
        } else {
            None
        }
    }
}

impl PartialEq for SemverVersion {
    fn eq(&self, other: &Self) -> bool {
        self.major == other.major && self.minor == other.minor && self.patch == other.patch
    }
}

impl Eq for SemverVersion {}

impl SemverVersion {
    pub fn new(version_number: &str) -> Result<SemverVersion, String> {
        debug_assert!(version_number == version_number.trim_start());
        let Some(first_digit_index) = version_number
            .trim_start()
            .find(|val: char| char::is_ascii_digit(&val))
        else {
            return Err(String::from("Invalid semver string"));
        };

        let mut version_number = version_number;
        if first_digit_index != 0 {
            let prefix = version_number[..first_digit_index].trim();
            if prefix == "=" || prefix == "^" {
                version_number = &version_number[first_digit_index..];
            } else if let Some(first_character) = prefix.get(0..1) {
                if "~<>=^".contains(first_character) {
                    return Err(String::from(
                        "~,<,>,<=, >= and multiple value version prefixes not yet supported",
                    ));
                }
                return Err(String::from("Invalid semver string"));
            } else {
                return Err(String::from("Invalid semver string"));
            }
        };

        if let Some((major, rest)) = version_number.split_once('.') {
            if let Ok(major) = major.parse::<u32>() {
                if let Some((minor, patch)) = rest.split_once('.') {
                    if let Ok(minor) = minor.parse::<u32>() {
                        if let Ok(patch) = patch.parse::<u32>() {
                            return Ok(SemverVersion {
                                major,
                                minor: Some(minor),
                                patch: Some(patch),
                            });
                        }
                    }
                    return Err(String::from("Invalid semver string"));
                }
                if let Ok(minor) = rest.parse::<u32>() {
                    return Ok(SemverVersion {
                        major,
                        minor: Some(minor),
                        patch: None,
                    });
                }
            }
            return Err(String::from("Invalid semver string"));
        } else if let Ok(major) = version_number.parse::<u32>() {
            return Ok(SemverVersion {
                major,
                minor: None,
                patch: None,
            });
        }
        Err(String::from("Invalid semver string"))
    }

    pub fn change_type(&self, other: &Self) -> SemverChange {
        debug_assert!(self.minor.is_some() || self.patch.is_none());
        debug_assert!(other.minor.is_some() || other.patch.is_none());
        if self.major != other.major {
            return SemverChange::Major;
        }
        if let (Some(self_minor), Some(other_minor)) = (self.minor, other.minor) {
            if self_minor != other_minor {
                if self.major > 0 {
                    return SemverChange::Minor;
                }
                return SemverChange::Major;
            }
            if let (Some(self_patch), Some(other_patch)) = (self.patch, other.patch) {
                if self_patch != other_patch {
                    if self.major > 0 {
                        return SemverChange::Patch;
                    }
                    if self_minor > 0 {
                        return SemverChange::Minor;
                    }
                    return SemverChange::Major;
                }
                return SemverChange::None;
            }
        }

        SemverChange::Unknown
    }
}

#[cfg(test)]
mod tests {
    use crate::domain::SemverVersion;

    use super::SemverChange;

    #[test]
    fn fmt_semver_change_displays_expected_values() {
        // act
        let result = format!(
            "Major: {}\nMinor: {}\nPatch: {}\nNone: {}\nUnknown: {}",
            SemverChange::Major,
            SemverChange::Minor,
            SemverChange::Patch,
            SemverChange::None,
            SemverChange::Unknown
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
                major: 1,
                minor: Some(2),
                patch: Some(3)
            }
        );
        assert_eq!(
            SemverVersion::new("1.2").unwrap(),
            SemverVersion {
                major: 1,
                minor: Some(2),
                patch: None
            }
        );
        assert_eq!(
            SemverVersion::new("1").unwrap(),
            SemverVersion {
                major: 1,
                minor: None,
                patch: None
            }
        );
    }

    #[test]
    fn semver_version_catches_invalid_semver_strings() {
        // assert
        let expected_error = String::from("Invalid semver string");
        assert_eq!(SemverVersion::new("1..3").unwrap_err(), expected_error);
        assert_eq!(SemverVersion::new("1.").unwrap_err(), expected_error);
        assert_eq!(SemverVersion::new("xyz").unwrap_err(), expected_error);
        assert_eq!(SemverVersion::new(".2").unwrap_err(), expected_error);

        let expected_error =
            String::from("~,<,>,<=, >= and multiple value version prefixes not yet supported");
        assert_eq!(SemverVersion::new(">2.1.3").unwrap_err(), expected_error);
    }

    #[test]
    fn change_type_returns_expected_values() {
        // assert
        assert_eq!(
            SemverVersion::new("1.2.3")
                .unwrap()
                .change_type(&SemverVersion::new("1.2.3").unwrap()),
            SemverChange::None
        );
        assert_eq!(
            SemverVersion::new("1.2.3")
                .unwrap()
                .change_type(&SemverVersion::new("2.2.3").unwrap()),
            SemverChange::Major
        );
        assert_eq!(
            SemverVersion::new("0.2.3")
                .unwrap()
                .change_type(&SemverVersion::new("0.3.3").unwrap()),
            SemverChange::Major
        );
        assert_eq!(
            SemverVersion::new("0.0.3")
                .unwrap()
                .change_type(&SemverVersion::new("0.0.4").unwrap()),
            SemverChange::Major
        );
        assert_eq!(
            SemverVersion::new("1.2.3")
                .unwrap()
                .change_type(&SemverVersion::new("1.3.3").unwrap()),
            SemverChange::Minor
        );
        assert_eq!(
            SemverVersion::new("0.1.2")
                .unwrap()
                .change_type(&SemverVersion::new("0.1.3").unwrap()),
            SemverChange::Minor
        );
        assert_eq!(
            SemverVersion::new("1.2.3")
                .unwrap()
                .change_type(&SemverVersion::new("1.2.4").unwrap()),
            SemverChange::Patch
        );
        assert_eq!(
            SemverVersion::new("1.2.3")
                .unwrap()
                .change_type(&SemverVersion::new("1").unwrap()),
            SemverChange::Unknown
        );
        assert_eq!(
            SemverVersion::new("1.2.3")
                .unwrap()
                .change_type(&SemverVersion::new("1.2").unwrap()),
            SemverChange::Unknown
        );
        assert_eq!(
            SemverVersion::new("1.2")
                .unwrap()
                .change_type(&SemverVersion::new("1").unwrap()),
            SemverChange::Unknown
        );
    }
}
