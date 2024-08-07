use regex::Regex;
use ring::digest::Digest;
use std::ffi::OsStr;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::{env, io};
use strum::Display;

#[derive(Debug, Eq, PartialEq, Copy, Clone, Display)]
#[strum(serialize_all = "camelCase")]
pub enum TargetOS {
  Macos,
  Windows,
  Linux,
}

#[derive(Debug, Clone)]
struct Config {
  os: TargetOS,
  features: &'static str,
  config: &'static str,
}

impl Config {
  fn zip_name(&self) -> String {
    format!(
      "libfreenect2-{}-{}-{}.zip",
      self.os, self.features, self.config
    )
  }
}

impl TargetOS {
  pub fn new() -> anyhow::Result<Self> {
    match env::var("CARGO_CFG_TARGET_OS")?.as_str() {
      "macos" => Ok(TargetOS::Macos),
      "windows" => Ok(TargetOS::Windows),
      "linux" => Ok(TargetOS::Linux),
      other => Err(other.to_string()),
    }
    .map_err(|e| anyhow::anyhow!("Unsupported OS: {e}"))
  }
}

fn get_lib_name(path: &Path, os: TargetOS) -> Option<&str> {
  if os == TargetOS::Windows {
    if path.extension()? != OsStr::new("lib") {
      return None;
    }

    path.file_stem()?.to_str()
  } else {
    if path.extension()? != OsStr::new("a") {
      return None;
    }

    let filename = path.file_stem()?.to_str()?;
    filename.strip_prefix("lib")
  }
}

fn sha256_digest(mut reader: impl io::Read) -> io::Result<Digest> {
  use ring::digest::{Context, SHA256};

  let mut context = Context::new(&SHA256);
  let mut buffer = [0; 8 * 1024];

  loop {
    let count = reader.read(&mut buffer)?;
    if count == 0 {
      break;
    }

    context.update(&buffer[..count]);
  }

  Ok(context.finish())
}

fn verify_sha256_of_file(path: &Path, expected_hex: &str) -> anyhow::Result<()> {
  let file = File::open(path)?;
  let sha256 = sha256_digest(file)?;
  let actual_hex = hex::encode(sha256.as_ref());
  anyhow::ensure!(
    actual_hex == expected_hex,
    "{:?}: sha256 does not match (actual: {}, expected: {})",
    path,
    actual_hex,
    expected_hex
  );

  Ok(())
}

fn get_sha256_for_filename(filename: &str, user: &str, repo: &str) -> Option<(String, String)> {
  let releases = tokio::runtime::Builder::new_current_thread()
    .enable_all()
    .build()
    .unwrap()
    .block_on(async move {
      let octocrab = match env::var("GITHUB_TOKEN") {
        Ok(token) => octocrab::Octocrab::builder()
          .personal_token(token)
          .build()
          .unwrap()
          .into(),
        Err(_) => octocrab::instance(),
      };

      octocrab
        .repos(user, repo)
        .releases()
        .list()
        .send()
        .await
        .unwrap()
        .items
    });

  for release in releases {
    if release.name.is_some() && release.name? == "libfreenect2" && release.body.is_some() {
      let body = release.body.unwrap();
      let checksums_str = Regex::new(r"## SHA256 Checksums\r?\n```([^`]*)```")
        .ok()?
        .captures(&body)?
        .get(1)?
        .as_str();

      let hash = Regex::new(r"\r?\n")
        .unwrap()
        .split(checksums_str.trim())
        .filter_map(|line| {
          let mut line_component_iter = line.trim().split(' ');
          let sha256 = line_component_iter.next()?.trim();
          let f_name = line_component_iter.next()?.strip_prefix('*')?;

          if f_name == filename {
            Some(sha256.to_owned())
          } else {
            None
          }
        })
        .next();

      if hash.is_none() {
        continue;
      }

      let url = release
        .assets
        .iter()
        .filter(|asset| asset.name == filename)
        .map(|asset| asset.url.to_string())
        .next();

      if let Some(url) = url {
        return Some((hash.unwrap(), url));
      }
    }
  }

  None
}

fn download(url: &str, path: &Path) -> anyhow::Result<()> {
  let file = File::create(path)?;
  let mut builder = attohttpc::get(url);
  if let Ok(token) = env::var("GITHUB_TOKEN") {
    builder = builder.header("Authorization", format!("token {}", token));
  }

  builder
    .header("Accept", "application/octet-stream")
    .send()?
    .error_for_status()?
    .write_to(file)
    .map(|_| ())
    .map_err(Into::into)
}

pub struct DownloadedLibrary {
  pub include_path: PathBuf,
}

pub struct ZippedLibrary {
  repo_owner: String,
  repo_name: String,
  local_path: String,
}

impl ZippedLibrary {
  pub fn new(repo_owner: &str, repo_name: &str, local_path: &str) -> Self {
    Self {
      repo_owner: repo_owner.to_owned(),
      repo_name: repo_name.to_owned(),
      local_path: local_path.to_owned(),
    }
  }

  pub fn download(self) -> anyhow::Result<DownloadedLibrary> {
    let out_dir = PathBuf::from(env::var("OUT_DIR")?);
    let os = TargetOS::new()?;

    if os == TargetOS::Windows {
      let target_env = env::var("CARGO_CFG_TARGET_ENV")?;
      anyhow::ensure!(
        target_env == "msvc",
        "Unsupported Environment ABI: {}",
        target_env
      );
    }

    println!("cargo:rerun-if-env-changed={}", self.local_path);
    let libfreenect2_path = if let Ok(path_from_env) = env::var(&self.local_path) {
      println!("cargo:rerun-if-changed={}", path_from_env);
      PathBuf::from(path_from_env)
    } else {
      let features = if cfg!(feature = "opencl") && cfg!(feature = "opengl") {
        "all"
      } else if cfg!(feature = "opencl") {
        "opencl"
      } else if cfg!(feature = "opengl") {
        "opengl"
      } else {
        panic!("At least one of 'opencl' or 'opengl' must be enabled")
      };

      let config = match env::var("PROFILE").unwrap().as_str() {
        "debug" => "debug",
        "release" => "release",
        rest => panic!("Unknown profile: {}", rest),
      };

      let config = Config {
        os,
        features,
        config,
      };

      let (sha256, url) = get_sha256_for_filename(
        config.zip_name().as_str(),
        &self.repo_owner,
        &self.repo_name,
      )
      .unwrap_or_else(|| {
        panic!(
          "No sha256 checksum found for filename: {}",
          config.zip_name().as_str()
        )
      });

      let libfreenect2_zip = out_dir.join(config.zip_name());
      if verify_sha256_of_file(libfreenect2_zip.as_path(), &sha256).is_err() {
        println!("Downloading {}", url);
        download(url.as_str(), libfreenect2_zip.as_path())?;
        println!("Verifying {:?}", libfreenect2_zip.as_path());
        verify_sha256_of_file(libfreenect2_zip.as_path(), &sha256)?;
      }

      let libfreenect2_extracted = out_dir.join("libfreenect2_extracted");
      let _ = std::fs::remove_dir_all(libfreenect2_extracted.as_path());
      println!("Extracting to {:?}", libfreenect2_extracted);
      zip_extract::extract(File::open(libfreenect2_zip)?, &libfreenect2_extracted, true)?;
      libfreenect2_extracted
    };

    let include_path = libfreenect2_path.join("include");
    let lib_path = libfreenect2_path.join("lib");

    println!(
      "cargo:rustc-link-search=native={}",
      lib_path.to_str().unwrap()
    );

    for file in std::fs::read_dir(lib_path)? {
      let file = file?;
      if !file.file_type()?.is_file() {
        continue;
      }

      let path = file.path();
      let lib_name = match get_lib_name(path.as_path(), os) {
        Some(lib_name) => lib_name,
        None => continue,
      };

      println!("cargo:rustc-link-lib={}", lib_name);
    }

    Ok(DownloadedLibrary { include_path })
  }
}
