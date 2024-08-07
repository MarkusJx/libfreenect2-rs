use crate::build_util::zipped_library::TargetOS;

pub fn link_os_libs() -> anyhow::Result<()> {
  for lib in get_os_libs()? {
    println!("cargo:rustc-link-lib={}", lib);
  }

  Ok(())
}

fn get_os_libs() -> anyhow::Result<Vec<&'static str>> {
  Ok(match TargetOS::new()? {
    TargetOS::Macos => {
      let mut libs = vec![
        "usb-1.0",
        "framework=CoreFoundation",
        "framework=VideoToolbox",
        "framework=CoreMedia",
        "framework=CoreVideo",
        "framework=IOKit",
        "framework=CoreGraphics",
        "framework=AppKit",
        "framework=StoreKit",
      ];

      if cfg!(feature = "opengl") {
        libs.append(&mut vec!["glfw3", "framework=OpenGL"]);
      }
      if cfg!(feature = "opencl") {
        libs.push("framework=OpenCL");
      }

      libs
    }
    TargetOS::Linux => {
      let mut libs = vec!["stdc++", "usb-1.0", "turbojpeg"];
      if cfg!(feature = "opengl") {
        libs.append(&mut vec!["glfw", "GL"]);
      }
      if cfg!(feature = "opencl") {
        libs.push("OpenCL");
      }

      libs
    }
    TargetOS::Windows => {
      let mut libs = vec!["user32", "gdi32", "shell32"];
      if cfg!(feature = "opengl") {
        libs.push("opengl32");
      }

      libs
    }
  })
}
