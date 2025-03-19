use std::{
    cmp::Ordering,
    fmt::{self},
    ops::Range,
};

use semver::{BuildMetadata, Comparator, Op, Prerelease, VersionReq};

#[cfg(test)]
mod tests;

/// Rust Cargo.toml accepted dependency version formats:
/// - `1.2.3`;
/// - `~1.2.3`;
/// - `*`, `1.*`, `1.2.*`;
/// - `>= 1.2.3`, `> 1.2.3`, `< 1.2.3`, `= 1.2.3`; and
/// - `>= 1.2, <1.5` (multiple version requirements for single dependency).
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
        Version::range_compare(&self.comparator_ranges(), &other.comparator_ranges())
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

        // let () = Self::error_if_comparator_operator_not_supported(&req)?;

        Ok(Self { req })
    }

    fn range_compare(a: &Range<semver::Version>, b: &Range<semver::Version>) -> Option<Ordering> {
        if a == b {
            Some(Ordering::Equal)
        } else if a.end <= b.start
            || (a.start < b.start && a.end == b.end)
            || (a.start == b.start && a.end < b.end)
        {
            Some(Ordering::Less)
        } else if a.start >= b.end
            || (a.start > b.start && a.end == b.end)
            || (a.start == b.start && a.end > b.end)
        {
            Some(Ordering::Greater)
        } else {
            None
        }
    }

    fn version_with_bumped_major(major: u64) -> semver::Version {
        semver::Version {
            major: major
                .checked_add(1)
                .unwrap_or_else(|| panic!("Unexpectedly high major version: `{major}`")),
            minor: 0,
            patch: 0,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        }
    }

    fn version_with_bumped_minor(major: u64, minor: u64) -> semver::Version {
        semver::Version {
            major,
            minor: minor
                .checked_add(1)
                .unwrap_or_else(|| panic!("Unexpectedly high minor version: `{minor}`")),
            patch: 0,
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        }
    }

    fn version_with_bumped_patch(major: u64, minor: u64, patch: u64) -> semver::Version {
        semver::Version {
            major,
            minor,
            patch: patch
                .checked_add(1)
                .unwrap_or_else(|| panic!("Unexpectedly high patch version: `{patch}`")),
            pre: Prerelease::EMPTY,
            build: BuildMetadata::EMPTY,
        }
    }

    fn caret_range(major: u64, minor: Option<u64>, patch: Option<u64>) -> Range<semver::Version> {
        match major {
            0 => {
                match minor {
                    Some(minor_version @ 0) => {
                        match patch {
                            Some(patch_version) => {
                                // ^0.0.K
                                Range {
                                    start: semver::Version::new(
                                        major,
                                        minor_version,
                                        patch_version,
                                    ),
                                    end: Self::version_with_bumped_patch(
                                        major,
                                        minor_version,
                                        patch_version,
                                    ),
                                }
                            }
                            None => {
                                // ^0.0
                                Range {
                                    start: semver::Version::new(major, minor_version, 0),
                                    end: Self::version_with_bumped_minor(major, minor_version),
                                }
                            }
                        }
                    }
                    Some(minor_version @ 1..) => {
                        match patch {
                            Some(patch_version) => {
                                // ^0.J.K (J > 0)
                                Range {
                                    start: semver::Version::new(
                                        major,
                                        minor_version,
                                        patch_version,
                                    ),
                                    end: Self::version_with_bumped_minor(major, minor_version),
                                }
                            }
                            None => {
                                // ^0.J (J > 0)
                                Range {
                                    start: semver::Version::new(major, minor_version, 0),
                                    end: Self::version_with_bumped_minor(major, minor_version),
                                }
                            }
                        }
                    }
                    // ^0
                    None => Range {
                        start: semver::Version::new(0, 0, 0),
                        end: semver::Version::new(1, 0, 0),
                    },
                }
            }
            1.. => {
                if let Some(minor_version) = minor {
                    if let Some(patch_version) = patch {
                        // ^I.J.K (I > 0)
                        Range {
                            start: semver::Version::new(major, minor_version, patch_version),
                            end: Self::version_with_bumped_major(major),
                        }
                    } else {
                        // ^I.J (I > 0, J > 0)
                        Range {
                            start: semver::Version::new(major, minor_version, 0),
                            end: Self::version_with_bumped_major(major),
                        }
                    }
                } else {
                    // ^I
                    Range {
                        start: semver::Version::new(major, 0, 0),
                        end: Self::version_with_bumped_major(major),
                    }
                }
            }
        }
    }

    fn exact_range(major: u64, minor: Option<u64>, patch: Option<u64>) -> Range<semver::Version> {
        if let Some(minor_version) = minor {
            if let Some(patch_version) = patch {
                // =I.J.K
                Range {
                    start: semver::Version::new(major, minor_version, patch_version),
                    end: Self::version_with_bumped_patch(major, minor_version, patch_version),
                }
            } else {
                // =I.J
                Range {
                    start: semver::Version::new(major, minor_version, 0),
                    end: Self::version_with_bumped_minor(major, minor_version),
                }
            }
        } else {
            // =I
            Range {
                start: semver::Version::new(major, 0, 0),
                end: Self::version_with_bumped_major(major),
            }
        }
    }

    fn greater_range(major: u64, minor: Option<u64>, patch: Option<u64>) -> Range<semver::Version> {
        let end = semver::Version::new(u64::MAX, u64::MAX, u64::MAX);
        if let Some(minor_version) = minor {
            if let Some(patch_version) = patch {
                // >I.J.K
                Range {
                    start: Self::version_with_bumped_patch(major, minor_version, patch_version),
                    end,
                }
            } else {
                // >I.J
                Range {
                    start: Self::version_with_bumped_minor(major, minor_version),
                    end,
                }
            }
        } else {
            // >I
            Range {
                start: Self::version_with_bumped_major(major),
                end,
            }
        }
    }

    fn greater_or_equal_range(
        major: u64,
        minor: Option<u64>,
        patch: Option<u64>,
    ) -> Range<semver::Version> {
        let end = semver::Version::new(u64::MAX, u64::MAX, u64::MAX);
        if let Some(minor_version) = minor {
            if let Some(patch_version) = patch {
                // >=I.J.K
                Range {
                    start: semver::Version::new(major, minor_version, patch_version),
                    end,
                }
            } else {
                // >=I.J
                Range {
                    start: semver::Version::new(major, minor_version, 0),
                    end,
                }
            }
        } else {
            // >=I
            Range {
                start: semver::Version::new(major, 0, 0),
                end,
            }
        }
    }

    fn less_range(major: u64, minor: Option<u64>, patch: Option<u64>) -> Range<semver::Version> {
        let start = semver::Version::new(0, 0, 0);
        if let Some(minor_version) = minor {
            if let Some(patch_version) = patch {
                // <I.J.K
                Range {
                    start,
                    end: semver::Version::new(major, minor_version, patch_version),
                }
            } else {
                // <I.J
                Range {
                    start,
                    end: semver::Version::new(major, minor_version, 0),
                }
            }
        } else {
            // <I
            Range {
                start,
                end: semver::Version::new(major, 0, 0),
            }
        }
    }

    fn less_or_equal_range(
        major: u64,
        minor: Option<u64>,
        patch: Option<u64>,
    ) -> Range<semver::Version> {
        let start = semver::Version::new(0, 0, 0);
        if let Some(minor_version) = minor {
            if let Some(patch_version) = patch {
                // <=I.J.K
                Range {
                    start,
                    end: Self::version_with_bumped_patch(major, minor_version, patch_version),
                }
            } else {
                // <=I.J
                Range {
                    start,
                    end: Self::version_with_bumped_minor(major, minor_version),
                }
            }
        } else {
            // <=I
            Range {
                start,
                end: Self::version_with_bumped_major(major),
            }
        }
    }

    fn tilde_range(major: u64, minor: Option<u64>, patch: Option<u64>) -> Range<semver::Version> {
        if let Some(minor_version) = minor {
            if let Some(patch_version) = patch {
                // ~I.J.K — equivalent to `>=I.J.K, <I.(J+1).0`
                Range {
                    start: semver::Version::new(major, minor_version, patch_version),
                    end: Self::version_with_bumped_minor(major, minor_version),
                }
            } else {
                // ~I.J — equivalent to `=I.J`
                Range {
                    start: semver::Version::new(major, minor_version, 0),
                    end: Self::version_with_bumped_minor(major, minor_version),
                }
            }
        } else {
            // ~I — equivalent to `=I`
            Range {
                start: semver::Version::new(major, 0, 0),
                end: Self::version_with_bumped_major(major),
            }
        }
    }

    fn wildcard_range(
        major: u64,
        minor: Option<u64>,
        patch: Option<u64>,
    ) -> Range<semver::Version> {
        debug_assert!(patch.is_none());

        if let Some(minor_version) = minor {
            // I.J.* — equivalent to `=I.J`
            Range {
                start: semver::Version::new(major, minor_version, 0),
                end: Self::version_with_bumped_minor(major, minor_version),
            }
        } else {
            // `I.*` or `I.*.*` — equivalent to `=I`
            Range {
                start: semver::Version::new(major, 0, 0),
                end: Self::version_with_bumped_major(major),
            }
        }
    }

    /// Returns a vector containing the ranges for each comparator.  Collapses all ranges into a
    /// single one if possible.  If collapsing to a single range is not possible, no attempt is
    /// made to collapse any pairs of elements, which could feasibly be collapsed.  Result is
    /// sorted by increasing range starts.
    fn comparator_ranges(&self) -> Range<semver::Version> {
        debug_assert!(!self.req.comparators.is_empty());

        let mut start = semver::Version::new(0, 0, 0);
        let mut end = semver::Version::new(u64::MAX, u64::MAX, u64::MAX);
        for comparator in &self.req.comparators {
            let Comparator {
                op,
                major,
                minor,
                patch,
                ..
            } = comparator;
            let range = match op {
                Op::Exact => Self::exact_range(*major, *minor, *patch),
                Op::Greater => Self::greater_range(*major, *minor, *patch),
                Op::GreaterEq => Self::greater_or_equal_range(*major, *minor, *patch),
                Op::Less => Self::less_range(*major, *minor, *patch),
                Op::LessEq => Self::less_or_equal_range(*major, *minor, *patch),
                Op::Tilde => Self::tilde_range(*major, *minor, *patch),
                Op::Caret => Self::caret_range(*major, *minor, *patch),
                Op::Wildcard => Self::wildcard_range(*major, *minor, *patch),
                _ => unimplemented!("Unsupported semver operator: `{op:?}`"),
            };

            match op {
                Op::Exact | Op::Tilde | Op::Caret | Op::Wildcard => {
                    if range.start > start {
                        start = range.start;
                    }
                    if range.end < end {
                        end = range.end;
                    }
                }
                Op::Greater | Op::GreaterEq => {
                    if range.start > start {
                        start = range.start;
                    }
                }
                Op::Less | Op::LessEq => {
                    if range.end < end {
                        end = range.end;
                    }
                }
                _ => unreachable!(
                    "Previous match statement should cause a panic on non-implemented operations."
                ),
            }
        }

        if end < start {
            log::error!(
                "Unexpected invalid range requirement: {:?}",
                self.req.comparators,
            );
        }
        Range { start, end }
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
