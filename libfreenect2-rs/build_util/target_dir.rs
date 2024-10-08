use std::env;
use std::ffi::OsStr;
use std::path::PathBuf;

/// Taken from cxx-build
pub(crate) enum TargetDir {
  Path(PathBuf),
  Unknown,
}

impl TargetDir {
  pub(crate) fn find() -> TargetDir {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    if let Some(target_dir) = env::var_os("CARGO_TARGET_DIR") {
      let target_dir = PathBuf::from(target_dir);
      return if target_dir.is_absolute() {
        TargetDir::Path(target_dir)
      } else {
        TargetDir::Unknown
      };
    }

    // fs::canonicalize on Windows produces UNC paths which cl.exe is unable to
    // handle in includes.
    // https://github.com/rust-lang/rust/issues/42869
    // https://github.com/alexcrichton/cc-rs/issues/169
    let mut also_try_canonical = cfg!(not(windows));

    let mut dir = out_dir.to_owned();
    loop {
      if dir.join(".rustc_info.json").exists()
        || dir.join("CACHEDIR.TAG").exists()
        || dir.file_name() == Some(OsStr::new("target"))
          && dir
            .parent()
            .map_or(false, |parent| parent.join("Cargo.toml").exists())
      {
        return TargetDir::Path(dir);
      }
      if dir.pop() {
        continue;
      }
      if also_try_canonical {
        if let Ok(canonical_dir) = out_dir.canonicalize() {
          dir = canonical_dir;
          also_try_canonical = false;
          continue;
        }
      }
      return TargetDir::Unknown;
    }
  }
}
