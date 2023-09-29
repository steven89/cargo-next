use semver::Version;
use std::{fs, io::Error as IoError, path::Path};
use thiserror::Error;
use toml_edit::{value, Document, Item, TomlError};

/// The error type of this crate.
#[derive(Debug, Error)]
pub enum Error {
    /// An error that occurred during the read and write operation of the
    /// `Cargo.toml` file.
    #[error("an io error occurred")]
    IoError(#[from] IoError),
    /// An error that occures while parsing a semver version string.
    #[error("An error occurred during version parsing")]
    SemverParseError(#[from] semver::Error),
    /// An error that occurred during the toml parsing.
    #[error("a toml parser error occurred")]
    ParseError(#[from] TomlError),
    /// An error that gets emitted if the `package.version` field has not the
    /// right type (String).
    #[error("the field {field:?} is not of type {ty:?}")]
    InvalidFieldType { field: String, ty: String },
}

/// An enum defining what types of increments can be done to a semver version.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum Increment {
    /// A major increment.
    Major,
    /// A minor increment.
    Minor,
    /// A patch increment.
    Patch,
}

pub fn get_package_version_str(path: impl AsRef<Path>) -> Result<String, Error> {
    let cargo_toml_content = fs::read_to_string(path.as_ref())?;
    let doc = cargo_toml_content.parse::<Document>()?;
    let item: &Item = &doc["package"]["version"];

    // This should be the case for valid Cargo.toml files.
    if let Some(s) = item.as_str() {
        Ok(s.to_string())
    } else {
        Err(Error::InvalidFieldType {
            field: "version".to_string(),
            ty: "string".to_string(),
        })
    }
}

/// Returns the version inside a `Cargo.toml` file.
///
/// # Arguments
///
/// - `path`: The path to the `Cargo.toml` file.
///
/// # Returns
///
/// The version as a `String` if it could be successfully extracted, otherwise
/// an error.
pub fn get_version(path: impl AsRef<Path>) -> Result<Version, Error> {
    let cargo_toml_content = fs::read_to_string(path.as_ref())?;
    let doc = cargo_toml_content.parse::<Document>()?;
    let item: &Item = &doc["package"]["version"];

    // This should be the case for valid Cargo.toml files.
    if let Some(s) = item.as_str() {
        Ok(Version::parse(s)?)
    } else {
        Err(Error::InvalidFieldType {
            field: "version".to_string(),
            ty: "string".to_string(),
        })
    }
}

/// Sets the version inside a `Cargo.toml` file.
///
/// # Arguments
///
/// - `path`: The path to the `Cargo.toml` file.
/// - `version`: The version to write into the file. Note that no checks are
///   done to see whether the value contains a valid semver version.
///
/// # Returns
///
/// An error if something went wrong during IO operations or parsing.
pub fn set_version(path: impl AsRef<Path>, version_str: impl AsRef<str>) -> Result<Version, Error> {
    let version = Version::parse(version_str.as_ref())?;
    let cargo_toml_content = fs::read_to_string(path.as_ref())?;
    let mut doc = cargo_toml_content.parse::<Document>()?;

    doc["package"]["version"] = value(&version.to_string());
    fs::write(path.as_ref(), doc.to_string())?;

    Ok(version)
}

/// Bumps the version inside a `Cargo.toml` file according to semver specs.
///
/// # Arguments
///
/// - `path`: The path to the `Cargo.toml` file.
/// - `type`: The type of bump. Either patch, minor or major.
///
/// # Returns
///
/// The new version or an error if something went wrong during IO operations.
pub fn bump_toml_version(path: impl AsRef<Path>, increment: Increment) -> Result<Version, Error> {
    let version_str = get_package_version_str(path.as_ref())?;
    let version = bump_version(&version_str, increment)?;
    set_version(path, &version.to_string())?;
    Ok(version)
}

pub fn bump_version(version_str: &str, increment: Increment) -> Result<Version, Error> {
    let mut version: Version = Version::parse(version_str)?;
    match increment {
        Increment::Major => version.bump_major(),
        Increment::Minor => version.bump_minor(),
        Increment::Patch => version.bump_patch(),
    }
    Ok(version)
}

trait SemVerExt {
    fn increment_major(&mut self);
    fn increment_minor(&mut self);
    fn increment_patch(&mut self);

    fn bump_major(&mut self);
    fn bump_minor(&mut self);
    fn bump_patch(&mut self);
}

impl SemVerExt for Version {
    fn increment_major(&mut self) {
        self.major += 1;
    }

    fn increment_minor(&mut self) {
        self.minor += 1;
    }

    fn increment_patch(&mut self) {
        self.patch += 1;
    }

    fn bump_major(&mut self) {
        self.major += 1;
        self.minor = 0;
        self.patch = 0;
    }

    fn bump_minor(&mut self) {
        self.minor += 1;
        self.patch = 0;
    }

    fn bump_patch(&mut self) {
        self.patch += 1;
    }
}

#[cfg(test)]
mod tests {
    use crate::{bump_version, Increment};

    #[test]
    fn test_version_bump() {
        const BASE_VERSION: &str = "0.1.0";
        let mut v = bump_version(BASE_VERSION, Increment::Patch).unwrap();
        assert_eq!(&v.to_string(), "0.1.1");
        v = bump_version(&v.to_string(), Increment::Minor).unwrap();
        assert_eq!(&v.to_string(), "0.2.0");
        v = bump_version(&v.to_string(), Increment::Patch).unwrap();
        v = bump_version(&v.to_string(), Increment::Patch).unwrap();
        assert_eq!(&v.to_string(), "0.2.2");
        v = bump_version(&v.to_string(), Increment::Major).unwrap();
        assert_eq!(&v.to_string(), "1.0.0");
        v = bump_version(&v.to_string(), Increment::Minor).unwrap();
        assert_eq!(&v.to_string(), "1.1.0");
        v = bump_version(&v.to_string(), Increment::Patch).unwrap();
        assert_eq!(&v.to_string(), "1.1.1");
    }
}
