use crate::ffi;

#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum FrameType {
  Color,
  Depth,
  Ir,
}

impl From<ffi::libfreenect2::FrameType> for FrameType {
  fn from(t: ffi::libfreenect2::FrameType) -> Self {
    match t {
      ffi::libfreenect2::FrameType::Color => FrameType::Color,
      ffi::libfreenect2::FrameType::Ir => FrameType::Ir,
      ffi::libfreenect2::FrameType::Depth => FrameType::Depth,
      _ => unreachable!(),
    }
  }
}
