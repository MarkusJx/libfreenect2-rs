fn main() {
  build(&["libfreenect2", "frame", "freenect2_device", "config"]);

  println!("cargo:rerun-if-changed=src/ffi.rs");

  println!(
    "cargo:rustc-link-search=native=C:\\Users\\marku\\Desktop\\libfreenect2\\build\\install\\lib"
  );
  println!("cargo:rustc-link-search=native=C:\\Users\\marku\\Desktop\\libfreenect2\\depends\\libusb\\MS64\\static");
  println!("cargo:rustc-link-search=native=C:\\Users\\marku\\Desktop\\libfreenect2\\depends\\glfw\\lib-vc2022");
  println!("cargo:rustc-link-search=native=C:\\Users\\marku\\Desktop\\libfreenect2\\depends\\libjpeg-turbo64\\lib");
  println!("cargo:rustc-link-search=native=C:\\Users\\marku\\Desktop\\libfreenect2\\depends\\opencl\\lib\\x86_64");

  println!("cargo:rustc-link-lib=freenect2static");
  println!("cargo:rustc-link-lib=libusb-1.0");
  println!("cargo:rustc-link-lib=glfw3");
  println!("cargo:rustc-link-lib=jpeg-static");
  println!("cargo:rustc-link-lib=turbojpeg-static");
  println!("cargo:rustc-link-lib=opengl32");
  println!("cargo:rustc-link-lib=user32");
  println!("cargo:rustc-link-lib=gdi32");
  println!("cargo:rustc-link-lib=shell32");
  println!("cargo:rustc-link-lib=OpenCL");
}

fn build(files: &[&str]) {
  let mut build = cxx_build::bridge("src/ffi.rs");
  build
    .include("../ffi/include")
    .include("C:\\Users\\marku\\Desktop\\libfreenect2\\build\\install\\include")
    .std("c++17");

  for file in files {
    build.file(format!("../ffi/src/{file}.cpp"));
    println!("cargo:rerun-if-changed=ffi/src/{file}.cpp");
    println!("cargo:rerun-if-changed=ffi/include/{file}.hpp");
  }

  build.compile("libfreenect2_ffi");
}
