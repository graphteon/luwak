use std::collections::HashMap;
use std::fs;

use anyhow::Context;
use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use tar::Builder;

use crate::testdata_path;

pub static CUSTOM_NPM_PACKAGE_CACHE: Lazy<CustomNpmPackageCache> =
  Lazy::new(CustomNpmPackageCache::default);

struct CustomNpmPackage {
  pub registry_file: String,
  pub tarballs: HashMap<String, Vec<u8>>,
}

/// Creates tarballs and a registry json file for npm packages
/// in the `testdata/npm/registry/@denotest` directory.
#[derive(Default)]
pub struct CustomNpmPackageCache(Mutex<HashMap<String, CustomNpmPackage>>);

impl CustomNpmPackageCache {
  pub fn tarball_bytes(
    &self,
    name: &str,
    version: &str,
  ) -> Result<Option<Vec<u8>>> {
    Ok(
      self
        .get_package_property(name, |p| p.tarballs.get(version).cloned())?
        .flatten(),
    )
  }

  pub fn registry_file(&self, name: &str) -> Result<Option<Vec<u8>>> {
    self.get_package_property(name, |p| p.registry_file.as_bytes().to_vec())
  }

  fn get_package_property<TResult>(
    &self,
    package_name: &str,
    func: impl FnOnce(&CustomNpmPackage) -> TResult,
  ) -> Result<Option<TResult>> {
    // it's ok if multiple threads race here as they will do the same work twice
    if !self.0.lock().contains_key(package_name) {
      match get_npm_package(package_name)? {
        Some(package) => {
          self.0.lock().insert(package_name.to_string(), package);
        }
        None => return Ok(None),
      }
    }
    Ok(self.0.lock().get(package_name).map(func))
  }
}

fn get_npm_package(package_name: &str) -> Result<Option<CustomNpmPackage>> {
  use ring::digest::Context;
  use ring::digest::SHA512;

  let package_folder = testdata_path().join("npm/registry").join(package_name);
  if !package_folder.exists() {
    return Ok(None);
  }

  // read all the package's versions
  let mut tarballs = HashMap::new();
  let mut versions = serde_json::Map::new();
  for entry in fs::read_dir(&package_folder)? {
    let entry = entry?;
    let file_type = entry.file_type()?;
    if !file_type.is_dir() {
      continue;
    }
    let version = entry.file_name().to_string_lossy().to_string();
    let version_folder = package_folder.join(&version);

    // create the tarball
    let mut tarball_bytes = Vec::new();
    {
      let mut encoder =
        GzEncoder::new(&mut tarball_bytes, Compression::default());
      {
        let mut builder = Builder::new(&mut encoder);
        builder
          .append_dir_all("package", &version_folder)
          .with_context(|| {
            format!(
              "Error adding tarball for directory: {}",
              version_folder.display()
            )
          })?;
        builder.finish()?;
      }
      encoder.finish()?;
    }

    // get tarball hash
    let mut hash_ctx = Context::new(&SHA512);
    hash_ctx.update(&tarball_bytes);
    let digest = hash_ctx.finish();
    let tarball_checksum = base64::encode(digest.as_ref()).to_lowercase();

    // create the registry file JSON for this version
    let mut dist = serde_json::Map::new();
    dist.insert(
      "integrity".to_string(),
      format!("sha512-{}", tarball_checksum).into(),
    );
    dist.insert("shasum".to_string(), "dummy-value".into());
    dist.insert(
      "tarball".to_string(),
      format!(
        "http://localhost:4545/npm/registry/{}/{}.tgz",
        package_name, version
      )
      .into(),
    );

    tarballs.insert(version.clone(), tarball_bytes);
    let package_json_path = version_folder.join("package.json");
    let package_json_text = fs::read_to_string(&package_json_path)
      .with_context(|| {
        format!(
          "Error reading package.json at {}",
          package_json_path.display()
        )
      })?;
    let mut version_info: serde_json::Map<String, serde_json::Value> =
      serde_json::from_str(&package_json_text)?;
    version_info.insert("dist".to_string(), dist.into());
    versions.insert(version, version_info.into());
  }

  // create the registry file for this package
  let mut registry_file = serde_json::Map::new();
  registry_file.insert("name".to_string(), package_name.to_string().into());
  registry_file.insert("versions".to_string(), versions.into());
  Ok(Some(CustomNpmPackage {
    registry_file: serde_json::to_string(&registry_file).unwrap(),
    tarballs,
  }))
}
