use std::{cmp::Ordering, ops::Range};

use semver::{Comparator, Op, Prerelease, VersionReq};

use super::Change;
use crate::domain::{semver::Version, SemverVersion};

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
fn format_version_displays_expected_values() {
    // arrange
    let version = Version::new("~7.3.7").unwrap();

    // act
    let result = format!("{version}");

    // assert
    assert_eq!(result, String::from("~7.3.7"));

    // arrange
    let version = Version::new("8.8.*").unwrap();

    // act
    let result = format!("{version}");

    // assert
    assert_eq!(result, String::from("8.8.*"));

    // arrange
    let version = Version::new("4.*").unwrap();

    // act
    let result = format!("{version}");

    // assert
    assert_eq!(result, String::from("4.*"));

    // arrange
    let version = Version::new("8.*.*").unwrap();

    // act
    let result = format!("{version}");

    // assert
    assert_eq!(result, String::from("8.*"));
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
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new("1.2.3")
            .unwrap()
            .partial_cmp(&SemverVersion::new("1").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new("1.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new("1").unwrap()),
        Some(Ordering::Greater)
    );
}

#[test]
fn semver_version_applies_partial_order_as_expected_for_caret_requirements() {
    // assert
    assert!(SemverVersion::new("^1.2.3").unwrap() < SemverVersion::new("^1.2.4").unwrap());
    assert!(SemverVersion::new("^1.2.3").unwrap() < SemverVersion::new("^1.3.2").unwrap());
    assert!(SemverVersion::new("^1.2.3").unwrap() < SemverVersion::new("^2.1.2").unwrap());
    assert!(SemverVersion::new("^1.2").unwrap() < SemverVersion::new("^1.3").unwrap());
    assert!(SemverVersion::new("^1.2").unwrap() < SemverVersion::new("^2.1").unwrap());
    assert!(SemverVersion::new("^10").unwrap() < SemverVersion::new("^200").unwrap());
    assert_eq!(
        SemverVersion::new("^1.2.3")
            .unwrap()
            .partial_cmp(&SemverVersion::new("^1.2").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new("^1.2.3")
            .unwrap()
            .partial_cmp(&SemverVersion::new("^1").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new("^1.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new("^1").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new("^1.2.4")
            .unwrap()
            .partial_cmp(&SemverVersion::new("^1.2.3").unwrap()),
        Some(Ordering::Greater)
    );
    assert!(SemverVersion::new("^1.3.2").unwrap() > SemverVersion::new("^1.2.3").unwrap());
    assert!(SemverVersion::new("^2.1.2").unwrap() > SemverVersion::new("^1.2.3").unwrap());
    assert!(SemverVersion::new("^1.3").unwrap() > SemverVersion::new("^1.2").unwrap());
    assert!(SemverVersion::new("^2.1").unwrap() > SemverVersion::new("^1.2").unwrap());
    assert!(SemverVersion::new("^200").unwrap() > SemverVersion::new("^10").unwrap());
    assert_eq!(
        SemverVersion::new("^1.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new("^1.2.3").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new("^1")
            .unwrap()
            .partial_cmp(&SemverVersion::new("^1.2.3").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new("^1")
            .unwrap()
            .partial_cmp(&SemverVersion::new("^1.2").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new("^0.0")
            .unwrap()
            .partial_cmp(&SemverVersion::new("^0").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new("^0.0")
            .unwrap()
            .partial_cmp(&SemverVersion::new("^0.1").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new("^0.0")
            .unwrap()
            .partial_cmp(&SemverVersion::new("^0.0.1").unwrap()),
        None
    );
    assert_eq!(
        SemverVersion::new("^0.1.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new("^0.1").unwrap()),
        Some(Ordering::Greater)
    );
}

#[test]
fn semver_version_applies_partial_order_as_expected_for_exact_requirements() {
    // assert
    assert!(SemverVersion::new("=1.2.3").unwrap() < SemverVersion::new("=1.2.4").unwrap());
    assert!(SemverVersion::new("=1.2.3").unwrap() < SemverVersion::new("=1.3.2").unwrap());
    assert!(SemverVersion::new("=1.2.3").unwrap() < SemverVersion::new("=2.1.2").unwrap());
    assert!(SemverVersion::new("=1.2").unwrap() < SemverVersion::new("=1.3").unwrap());
    assert!(SemverVersion::new("=1.2").unwrap() < SemverVersion::new("=2.1").unwrap());
    assert!(SemverVersion::new("=10").unwrap() < SemverVersion::new("=200").unwrap());
    assert_eq!(
        SemverVersion::new("=1.2.3")
            .unwrap()
            .partial_cmp(&SemverVersion::new("=1.2").unwrap()),
        None
    );
    assert_eq!(
        SemverVersion::new("=1.2.3")
            .unwrap()
            .partial_cmp(&SemverVersion::new("=1").unwrap()),
        None
    );
    assert_eq!(
        SemverVersion::new("=1.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new("=1").unwrap()),
        None
    );
    assert_eq!(
        SemverVersion::new("=1.2.4")
            .unwrap()
            .partial_cmp(&SemverVersion::new("=1.2.3").unwrap()),
        Some(Ordering::Greater)
    );
    assert!(SemverVersion::new("=1.3.2").unwrap() > SemverVersion::new("=1.2.3").unwrap());
    assert!(SemverVersion::new("=2.1.2").unwrap() > SemverVersion::new("=1.2.3").unwrap());
    assert!(SemverVersion::new("=1.3").unwrap() > SemverVersion::new("=1.2").unwrap());
    assert!(SemverVersion::new("=2.1").unwrap() > SemverVersion::new("=1.2").unwrap());
    assert!(SemverVersion::new("=200").unwrap() > SemverVersion::new("=10").unwrap());
    assert_eq!(
        SemverVersion::new("=1.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new("=1.2.3").unwrap()),
        None
    );
    assert_eq!(
        SemverVersion::new("=1")
            .unwrap()
            .partial_cmp(&SemverVersion::new("=1.2.3").unwrap()),
        None
    );
    assert_eq!(
        SemverVersion::new("=1")
            .unwrap()
            .partial_cmp(&SemverVersion::new("=1.2").unwrap()),
        None
    );
}

#[test]
fn semver_version_applies_partial_order_as_expected_for_greater_requirements() {
    // assert
    assert!(SemverVersion::new(">1.2.3").unwrap() < SemverVersion::new(">1.2.4").unwrap());
    assert!(SemverVersion::new(">1.2.3").unwrap() < SemverVersion::new(">1.3.2").unwrap());
    assert!(SemverVersion::new(">1.2.3").unwrap() < SemverVersion::new(">2.1.2").unwrap());
    assert!(SemverVersion::new(">1.2").unwrap() < SemverVersion::new(">1.3").unwrap());
    assert!(SemverVersion::new(">1.2").unwrap() < SemverVersion::new(">2.1").unwrap());
    assert!(SemverVersion::new(">10").unwrap() < SemverVersion::new(">200").unwrap());
    assert_eq!(
        SemverVersion::new(">1.2.3")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">1.2").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new(">1.2.3")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">1").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new(">1.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">1").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new(">1.2.4")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">1.2.3").unwrap()),
        Some(Ordering::Greater)
    );
    assert!(SemverVersion::new(">1.3.2").unwrap() > SemverVersion::new(">1.2.3").unwrap());
    assert!(SemverVersion::new(">2.1.2").unwrap() > SemverVersion::new(">1.2.3").unwrap());
    assert!(SemverVersion::new(">1.3").unwrap() > SemverVersion::new(">1.2").unwrap());
    assert!(SemverVersion::new(">2.1").unwrap() > SemverVersion::new(">1.2").unwrap());
    assert!(SemverVersion::new(">200").unwrap() > SemverVersion::new(">10").unwrap());
    assert_eq!(
        SemverVersion::new(">1.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">1.2.3").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new(">1")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">1.2.3").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new(">1")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">1.2").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new("=1.2.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">1.2.2").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new("=10")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">9.0.9").unwrap()),
        None
    );
}

#[test]
fn semver_version_applies_partial_order_as_expected_for_greater_or_equal_requirements() {
    // assert
    assert!(SemverVersion::new(">=1.2.3").unwrap() < SemverVersion::new(">=1.2.4").unwrap());
    assert!(SemverVersion::new(">=1.2.3").unwrap() < SemverVersion::new(">=1.3.2").unwrap());
    assert!(SemverVersion::new(">=1.2.3").unwrap() < SemverVersion::new(">=2.1.2").unwrap());
    assert!(SemverVersion::new(">=1.2").unwrap() < SemverVersion::new(">=1.3").unwrap());
    assert!(SemverVersion::new(">=1.2").unwrap() < SemverVersion::new(">=2.1").unwrap());
    assert!(SemverVersion::new(">=10").unwrap() < SemverVersion::new(">=200").unwrap());
    assert_eq!(
        SemverVersion::new(">=1.2.3")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">=1.2").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new(">=1.2.3")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">=1").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new(">=1.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">=1").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new(">=1.2.4")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">=1.2.3").unwrap()),
        Some(Ordering::Greater)
    );
    assert!(SemverVersion::new(">=1.3.2").unwrap() > SemverVersion::new(">=1.2.3").unwrap());
    assert!(SemverVersion::new(">=2.1.2").unwrap() > SemverVersion::new(">=1.2.3").unwrap());
    assert!(SemverVersion::new(">=1.3").unwrap() > SemverVersion::new(">=1.2").unwrap());
    assert!(SemverVersion::new(">=2.1").unwrap() > SemverVersion::new(">=1.2").unwrap());
    assert!(SemverVersion::new(">=200").unwrap() > SemverVersion::new(">=10").unwrap());
    assert_eq!(
        SemverVersion::new(">=1.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">=1.2.3").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new(">=1")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">=1.2.3").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new(">=1")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">=1.2").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new("=1.2.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">=1.2.2").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new("=10")
            .unwrap()
            .partial_cmp(&SemverVersion::new(">=9.0.9").unwrap()),
        None
    );
}

#[test]
fn semver_version_applies_partial_order_as_expected_for_less_requirements() {
    // assert
    assert!(SemverVersion::new("<1.2.3").unwrap() < SemverVersion::new("<1.2.4").unwrap());
    assert!(SemverVersion::new("<1.2.3").unwrap() < SemverVersion::new("<1.3.2").unwrap());
    assert!(SemverVersion::new("<1.2.3").unwrap() < SemverVersion::new("<2.1.2").unwrap());
    assert!(SemverVersion::new("<1.2").unwrap() < SemverVersion::new("<1.3").unwrap());
    assert!(SemverVersion::new("<1.2").unwrap() < SemverVersion::new("<2.1").unwrap());
    assert!(SemverVersion::new("<10").unwrap() < SemverVersion::new("<200").unwrap());
    assert_eq!(
        SemverVersion::new("<1.2.3")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<1.2").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new("<1.2.3")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<1").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new("<1.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<1").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new("<1.2.4")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<1.2.3").unwrap()),
        Some(Ordering::Greater)
    );
    assert!(SemverVersion::new("<1.3.2").unwrap() > SemverVersion::new("<1.2.3").unwrap());
    assert!(SemverVersion::new("<2.1.2").unwrap() > SemverVersion::new("<1.2.3").unwrap());
    assert!(SemverVersion::new("<1.3").unwrap() > SemverVersion::new("<1.2").unwrap());
    assert!(SemverVersion::new("<2.1").unwrap() > SemverVersion::new("<1.2").unwrap());
    assert!(SemverVersion::new("<200").unwrap() > SemverVersion::new("<10").unwrap());
    assert_eq!(
        SemverVersion::new("<1.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<1.2.3").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new("<1")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<1.2.3").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new("<1")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<1.2").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new("=1.2.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<1.2.2").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new("=10")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<9.0.9").unwrap()),
        Some(Ordering::Greater)
    );
}

#[test]
fn semver_version_applies_partial_order_as_expected_for_less_or_equal_requirements() {
    // assert
    assert!(SemverVersion::new("<=1.2.3").unwrap() < SemverVersion::new("<=1.2.4").unwrap());
    assert!(SemverVersion::new("<=1.2.3").unwrap() < SemverVersion::new("<=1.3.2").unwrap());
    assert!(SemverVersion::new("<=1.2.3").unwrap() < SemverVersion::new("<=2.1.2").unwrap());
    assert!(SemverVersion::new("<=1.2").unwrap() < SemverVersion::new("<=1.3").unwrap());
    assert!(SemverVersion::new("<=1.2").unwrap() < SemverVersion::new("<=2.1").unwrap());
    assert!(SemverVersion::new("<=10").unwrap() < SemverVersion::new("<=200").unwrap());
    assert_eq!(
        SemverVersion::new("<=1.2.3")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<=1.2").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new("<=1.2.3")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<=1").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new("<=1.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<=1").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new("<=1.2.4")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<=1.2.3").unwrap()),
        Some(Ordering::Greater)
    );
    assert!(SemverVersion::new("<=1.3.2").unwrap() > SemverVersion::new("<=1.2.3").unwrap());
    assert!(SemverVersion::new("<=2.1.2").unwrap() > SemverVersion::new("<=1.2.3").unwrap());
    assert!(SemverVersion::new("<=1.3").unwrap() > SemverVersion::new("<=1.2").unwrap());
    assert!(SemverVersion::new("<=2.1").unwrap() > SemverVersion::new("<=1.2").unwrap());
    assert!(SemverVersion::new("<=200").unwrap() > SemverVersion::new("<=10").unwrap());
    assert_eq!(
        SemverVersion::new("<=1.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<=1.2.3").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new("<=1")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<=1.2.3").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new("<=1")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<=1.2").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new("=1.2.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<=1.2.2").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new("=10")
            .unwrap()
            .partial_cmp(&SemverVersion::new("<=9.0.9").unwrap()),
        Some(Ordering::Greater)
    );
}

#[test]
fn semver_version_applies_partial_order_as_expected_for_tilde_requirements() {
    // assert
    assert!(SemverVersion::new("~1.2.3").unwrap() < SemverVersion::new("~1.2.4").unwrap());
    assert!(SemverVersion::new("~1.2.3").unwrap() < SemverVersion::new("~1.3.2").unwrap());
    assert!(SemverVersion::new("~1.2.3").unwrap() < SemverVersion::new("~2.1.2").unwrap());
    assert!(SemverVersion::new("~1.2").unwrap() < SemverVersion::new("~1.3").unwrap());
    assert!(SemverVersion::new("~1.2").unwrap() < SemverVersion::new("~2.1").unwrap());
    assert!(SemverVersion::new("~10").unwrap() < SemverVersion::new("~200").unwrap());
    assert_eq!(
        SemverVersion::new("~1.2.3")
            .unwrap()
            .partial_cmp(&SemverVersion::new("~1.2").unwrap()),
        Some(Ordering::Greater)
    );
    assert_eq!(
        SemverVersion::new("~1.2.3")
            .unwrap()
            .partial_cmp(&SemverVersion::new("~1").unwrap()),
        None
    );
    assert_eq!(
        SemverVersion::new("~1.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new("~1").unwrap()),
        None
    );
    assert_eq!(
        SemverVersion::new("~1.2.4")
            .unwrap()
            .partial_cmp(&SemverVersion::new("~1.2.3").unwrap()),
        Some(Ordering::Greater)
    );
    assert!(SemverVersion::new("~1.3.2").unwrap() > SemverVersion::new("~1.2.3").unwrap());
    assert!(SemverVersion::new("~2.1.2").unwrap() > SemverVersion::new("~1.2.3").unwrap());
    assert!(SemverVersion::new("~1.3").unwrap() > SemverVersion::new("~1.2").unwrap());
    assert_eq!(
        SemverVersion::new("~2.1")
            .unwrap()
            .partial_cmp(&SemverVersion::new("~2").unwrap()),
        None
    );
    assert!(SemverVersion::new("~200").unwrap() > SemverVersion::new("~10").unwrap());
    assert_eq!(
        SemverVersion::new("~1.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new("~1.2.3").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new("~1")
            .unwrap()
            .partial_cmp(&SemverVersion::new("~1.2.3").unwrap()),
        None
    );
    assert_eq!(
        SemverVersion::new("~1")
            .unwrap()
            .partial_cmp(&SemverVersion::new("~1.2").unwrap()),
        None
    );
    assert_eq!(
        SemverVersion::new("=1.2.2")
            .unwrap()
            .partial_cmp(&SemverVersion::new("~1.2.2").unwrap()),
        Some(Ordering::Less)
    );
    assert_eq!(
        SemverVersion::new("=10")
            .unwrap()
            .partial_cmp(&SemverVersion::new("~9.0.9").unwrap()),
        Some(Ordering::Greater)
    );
}

#[test]
fn semver_version_applies_partial_order_as_expected_for_wildcard_requirements() {
    // assert
    assert!(SemverVersion::new("1.2.*").unwrap() < SemverVersion::new("1.3.*").unwrap());
    assert!(SemverVersion::new("1.2.*").unwrap() < SemverVersion::new("2.1.*").unwrap());
    assert!(SemverVersion::new("10.*").unwrap() < SemverVersion::new("200.*").unwrap());
    assert_eq!(
        SemverVersion::new("1.2.*")
            .unwrap()
            .partial_cmp(&SemverVersion::new("1.*").unwrap()),
        None
    );
    assert!(SemverVersion::new("1.3.*").unwrap() > SemverVersion::new("1.2.*").unwrap());
    assert_eq!(
        SemverVersion::new("2.1.*")
            .unwrap()
            .partial_cmp(&SemverVersion::new("2.*").unwrap()),
        None
    );
    assert!(SemverVersion::new("200.*").unwrap() > SemverVersion::new("10.*").unwrap());
    assert_eq!(
        SemverVersion::new("1.*")
            .unwrap()
            .partial_cmp(&SemverVersion::new("1.2.*").unwrap()),
        None
    );
    assert_eq!(
        SemverVersion::new("=10")
            .unwrap()
            .partial_cmp(&SemverVersion::new("9.0.*").unwrap()),
        Some(Ordering::Greater)
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
        SemverVersion::new(">=1.2, <1.5").unwrap_err(),
        String::from("Multiple version requirement comparison is not yet implemented")
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

#[test]
fn version_range_returns_expected_value_for_caret_requirements() {
    // arrange
    let version = SemverVersion::new("^1.7.9").unwrap();

    // act
    let outcome = version.range();

    // assert
    assert_eq!(
        outcome,
        Range {
            start: semver::Version::new(1, 7, 9),
            end: semver::Version::new(2, 0, 0),
        },
    );

    // arrange
    let version = SemverVersion::new("^0.3.7").unwrap();

    // act
    let outcome = version.range();

    // assert
    assert_eq!(
        outcome,
        Range {
            start: semver::Version::new(0, 3, 7),
            end: semver::Version::new(0, 4, 0),
        },
    );

    // arrange
    let version = SemverVersion::new("^0.10").unwrap();

    // act
    let outcome = version.range();

    // assert
    assert_eq!(
        outcome,
        Range {
            start: semver::Version::new(0, 10, 0),
            end: semver::Version::new(0, 11, 0),
        },
    );

    // arrange
    let version = SemverVersion::new("^0.0.2").unwrap();

    // act
    let outcome = version.range();

    // assert
    assert_eq!(
        outcome,
        Range {
            start: semver::Version::new(0, 0, 2),
            end: semver::Version::new(0, 0, 3),
        },
    );

    // arrange
    let version = SemverVersion::new("^5.6").unwrap();

    // act
    let outcome = version.range();

    // assert
    assert_eq!(
        outcome,
        Range {
            start: semver::Version::new(5, 6, 0),
            end: semver::Version::new(6, 0, 0),
        },
    );

    // arrange
    let version = SemverVersion::new("^0.0").unwrap();

    // act
    let outcome = version.range();

    // assert
    assert_eq!(
        outcome,
        Range {
            start: semver::Version::new(0, 0, 0),
            end: semver::Version::new(0, 1, 0),
        },
    );

    // arrange
    let version = SemverVersion::new("^0").unwrap();

    // act
    let outcome = version.range();

    // assert
    assert_eq!(
        outcome,
        Range {
            start: semver::Version::new(0, 0, 0),
            end: semver::Version::new(1, 0, 0),
        },
    );

    // arrange
    let version = SemverVersion::new("^4").unwrap();

    // act
    let outcome = version.range();

    // assert
    assert_eq!(
        outcome,
        Range {
            start: semver::Version::new(4, 0, 0),
            end: semver::Version::new(5, 0, 0),
        },
    );
}

#[test]
fn version_range_returns_expected_value_for_exact_requirements() {
    // arrange
    let version = SemverVersion::new("=8.1.8").unwrap();

    // act
    let outcome = version.range();

    // assert
    assert_eq!(
        outcome,
        Range {
            start: semver::Version::new(8, 1, 8),
            end: semver::Version::new(8, 1, 9),
        },
    );

    // arrange
    let version = SemverVersion::new("=0.7").unwrap();

    // act
    let outcome = version.range();

    // assert
    assert_eq!(
        outcome,
        Range {
            start: semver::Version::new(0, 7, 0),
            end: semver::Version::new(0, 8, 0),
        },
    );

    // arrange
    let version = SemverVersion::new("=2").unwrap();

    // act
    let outcome = version.range();

    // assert
    assert_eq!(
        outcome,
        Range {
            start: semver::Version::new(2, 0, 0),
            end: semver::Version::new(3, 0, 0),
        },
    );
}
