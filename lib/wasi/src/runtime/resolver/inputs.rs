use std::{
    fmt::{self, Display, Formatter},
    fs::File,
    io::{BufRead, BufReader},
    path::{Path, PathBuf},
    str::FromStr,
};

use anyhow::Context;
use semver::{Version, VersionReq};
use sha2::{Digest, Sha256};
use url::Url;

use crate::runtime::resolver::{PackageId, SourceId};

/// A reference to *some* package somewhere that the user wants to run.
///
/// # Security Considerations
///
/// The [`PackageSpecifier::Path`] variant doesn't specify which filesystem a
/// [`Source`][source] will eventually query. Consumers of [`PackageSpecifier`]
/// should be wary of sandbox escapes.
///
/// [source]: crate::runtime::resolver::Source
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum PackageSpecifier {
    Registry {
        full_name: String,
        version: VersionReq,
    },
    Url(Url),
    /// A `*.webc` file on disk.
    Path(PathBuf),
}

impl FromStr for PackageSpecifier {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // TODO: Replace this with something more rigorous that can also handle
        // the locator field
        let (full_name, version) = match s.split_once('@') {
            Some((n, v)) => (n, v),
            None => (s, "*"),
        };

        let invalid_character = full_name
            .char_indices()
            .find(|(_, c)| !matches!(c, 'a'..='z' | 'A'..='Z' | '0'..='9' | '.'| '-'|'_' | '/'));
        if let Some((index, c)) = invalid_character {
            anyhow::bail!("Invalid character, {c:?}, at offset {index}");
        }

        let version = version
            .parse()
            .with_context(|| format!("Invalid version number, \"{version}\""))?;

        Ok(PackageSpecifier::Registry {
            full_name: full_name.to_string(),
            version,
        })
    }
}

impl Display for PackageSpecifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PackageSpecifier::Registry { full_name, version } => write!(f, "{full_name}@{version}"),
            PackageSpecifier::Url(url) => Display::fmt(url, f),
            PackageSpecifier::Path(path) => write!(f, "{}", path.display()),
        }
    }
}

/// A dependency constraint.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Dependency {
    pub alias: String,
    pub pkg: PackageSpecifier,
}

impl Dependency {
    pub fn package_name(&self) -> Option<&str> {
        match &self.pkg {
            PackageSpecifier::Registry { full_name, .. } => Some(full_name),
            _ => None,
        }
    }

    pub fn alias(&self) -> &str {
        &self.alias
    }

    pub fn version(&self) -> Option<&VersionReq> {
        match &self.pkg {
            PackageSpecifier::Registry { version, .. } => Some(version),
            _ => None,
        }
    }
}

/// Some metadata a [`Source`][source] can provide about a package without
/// needing to download the entire `*.webc` file.
///
/// [source]: crate::runtime::resolver::Source
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Summary {
    pub pkg: PackageInfo,
    pub dist: DistributionInfo,
}

impl Summary {
    pub fn package_id(&self) -> PackageId {
        PackageId {
            package_name: self.pkg.name.clone(),
            version: self.pkg.version.clone(),
            source: self.dist.source.clone(),
        }
    }
}

/// Information about a package's contents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackageInfo {
    /// The package's full name (i.e. `wasmer/wapm2pirita`).
    pub name: String,
    /// The package version.
    pub version: Version,
    /// Commands this package exposes to the outside world.
    pub commands: Vec<Command>,
    /// The name of a [`Command`] that should be used as this package's
    /// entrypoint.
    pub entrypoint: Option<String>,
    /// Any dependencies this package may have.
    pub dependencies: Vec<Dependency>,
}

/// Information used when retrieving a package.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DistributionInfo {
    /// A URL that can be used to download the `*.webc` file.
    pub webc: Url,
    /// A SHA-256 checksum for the `*.webc` file.
    pub webc_sha256: WebcHash,
    /// The [`Source`][source] this [`Summary`] came from.
    ///
    /// [source]: crate::runtime::resolver::Source
    pub source: SourceId,
}

/// The SHA-256 hash of a `*.webc` file.
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct WebcHash([u8; 32]);

impl WebcHash {
    pub fn from_bytes(bytes: [u8; 32]) -> Self {
        WebcHash(bytes)
    }

    pub fn for_file(path: impl AsRef<Path>) -> Result<Self, std::io::Error> {
        let mut hasher = Sha256::default();
        let mut reader = BufReader::new(File::open(path)?);

        loop {
            let buffer = reader.fill_buf()?;
            if buffer.is_empty() {
                break;
            }
            hasher.update(buffer);
            let bytes_read = buffer.len();
            reader.consume(bytes_read);
        }

        let hash = hasher.finalize().into();
        Ok(WebcHash::from_bytes(hash))
    }

    /// Generate a new [`WebcHash`] based on the SHA-256 hash of some bytes.
    pub fn sha256(webc: impl AsRef<[u8]>) -> Self {
        let webc = webc.as_ref();

        let mut hasher = Sha256::default();
        hasher.update(webc);
        WebcHash::from_bytes(hasher.finalize().into())
    }

    pub fn as_bytes(self) -> [u8; 32] {
        self.0
    }
}

impl From<[u8; 32]> for WebcHash {
    fn from(bytes: [u8; 32]) -> Self {
        WebcHash::from_bytes(bytes)
    }
}

impl Display for WebcHash {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for byte in self.0 {
            write!(f, "{byte:02X}")?;
        }

        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Command {
    pub name: String,
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn parse_some_package_specifiers() {
        let inputs = [
            (
                "first",
                PackageSpecifier::Registry {
                    full_name: "first".to_string(),
                    version: VersionReq::STAR,
                },
            ),
            (
                "namespace/package",
                PackageSpecifier::Registry {
                    full_name: "namespace/package".to_string(),
                    version: VersionReq::STAR,
                },
            ),
            (
                "namespace/package@1.0.0",
                PackageSpecifier::Registry {
                    full_name: "namespace/package".to_string(),
                    version: "1.0.0".parse().unwrap(),
                },
            ),
        ];

        for (src, expected) in inputs {
            let parsed = PackageSpecifier::from_str(src).unwrap();
            assert_eq!(parsed, expected);
        }
    }
}
