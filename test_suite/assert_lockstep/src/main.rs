use cargo_lock::{Lockfile, Package, Version};
use std::convert::Infallible;
use std::fmt;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str::FromStr;
use thiserror::Error;

#[derive(Debug, Error)]
enum TestError {
    #[error("Package \"{name}\" not found in the lockfile")]
    NoPackageFound { name: String },
    #[error("Package \"{first}\" and \"{second}\" had different versions in the lockfile")]
    VersionMismatch { first: PackageId, second: PackageId },
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Error parsing lockfile: {0}")]
    Lock(#[from] cargo_lock::Error),
}

impl From<Infallible> for TestError {
    fn from(value: Infallible) -> Self {
        match value {}
    }
}

#[derive(Debug)]
struct PackageId {
    name: String,
    version: Version,
}

impl PackageId {
    fn new(package: &Package) -> Self {
        Self {
            name: package.name.to_string(),
            version: package.version.clone(),
        }
    }
}

impl fmt::Display for PackageId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{name}@{version}",
            name = self.name,
            version = self.version,
        )
    }
}

fn find_package<'a>(lockfile: &'a Lockfile, package_name: &str) -> Result<&'a Package, TestError> {
    lockfile
        .packages
        .iter()
        .find(|package| package.name.as_str() == package_name)
        .ok_or_else(|| TestError::NoPackageFound {
            name: package_name.to_owned(),
        })
}

fn main() {
    if let Err(error) = main_inner() {
        panic!("{}", error);
    }
}

fn main_inner() -> Result<(), TestError> {
    let path = PathBuf::from_str(&std::env::args().nth(1).unwrap())?;

    Command::new("cargo")
        .arg("clean")
        .current_dir(&path)
        .stdout(Stdio::inherit())
        .output()?;

    Command::new("cargo")
        .arg("update")
        .current_dir(&path)
        .stdout(Stdio::inherit())
        .output()?;

    let lockfile = Lockfile::load(path.join("Cargo.lock"))?;

    let serde = find_package(&lockfile, "serde")?;

    println!("packages should match {id}", id = PackageId::new(&serde));

    let package_names = &["serde_derive"];

    let packages = package_names
        .iter()
        .map(|name| find_package(&lockfile, name));

    for package in packages {
        let package = package?;

        println!("discovered package {id}", id = PackageId::new(&package));

        if package.version != serde.version {
            return Err(TestError::VersionMismatch {
                first: PackageId::new(&serde),
                second: PackageId::new(&package),
            });
        }
    }

    Ok(())
}
