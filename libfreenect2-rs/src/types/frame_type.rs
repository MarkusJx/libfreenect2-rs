use crate::ffi;

/// The type of a Kinect frame.
#[derive(Debug, Ord, PartialOrd, Eq, PartialEq, Hash, Copy, Clone)]
pub enum FrameType {
  /// A color frame.
  Color,
  /// A depth frame.
  Depth,
  /// An infrared frame.
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
