use crate::frame_data::RGBX;
use std::fmt;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone, PartialEq, PartialOrd)]
/// A value in a frame.
pub enum FrameValue {
  /// Raw data. Contains `bytes_per_pixel` bytes.
  Raw(Vec<u8>),
  /// Invalid data.
  Invalid(u8),
  /// A 4-byte float.
  /// If the data is depth data, this is the depth in millimeters.
  /// If the data is <= 0, it is invalid.
  Float(f32),
  /// 4 bytes of R, G, B, and unused per pixel.
  RGBX(RGBX),
  /// 1 byte of gray per pixel.
  Gray(u8),
}

impl FrameValue {
  /// Expect the value to be raw data.
  /// Panics if the value is not raw data.
  pub fn expect_raw(&self) -> &Vec<u8> {
    match self {
      FrameValue::Raw(value) => value,
      _ => panic!("Expected raw value, found {}", self),
    }
  }

  /// Expect the value to be a float.
  /// Panics if the value is not a float.
  pub fn expect_float(&self) -> f32 {
    match self {
      FrameValue::Float(value) => *value,
      _ => panic!("Expected float value, found {}", self),
    }
  }

  /// Expect the value to be a RGBX value.
  /// Panics if the value is not a RGBX value.
  pub fn expect_rgbx(&self) -> &RGBX {
    match self {
      FrameValue::RGBX(value) => value,
      _ => panic!("Expected RGBX value, found {}", self),
    }
  }

  /// Expect the value to be a gray value.
  /// Panics if the valid is not a gray value.
  pub fn expect_gray(&self) -> u8 {
    match self {
      FrameValue::Gray(value) => *value,
      _ => panic!("Expected gray value, found {}", self),
    }
  }

  /// Get the raw data, if the value is raw data.
  /// Returns [`None`] if the value is not raw data.
  pub fn raw(&self) -> Option<&Vec<u8>> {
    match self {
      FrameValue::Raw(value) => Some(value),
      _ => None,
    }
  }

  /// Get the raw data, if the value is a float.
  /// Returns [`None`] if the value is not a float.
  pub fn float(&self) -> Option<f32> {
    match self {
      FrameValue::Float(value) => Some(*value),
      _ => None,
    }
  }

  /// Get the raw data, if the value is a RGBX value.
  /// Returns [`None`] if the value is not a RGBX value.
  pub fn rgbx(&self) -> Option<&RGBX> {
    match self {
      FrameValue::RGBX(value) => Some(value),
      _ => None,
    }
  }

  /// Get the raw data, if the value is a gray value.
  /// Returns [`None`] if the value is not a gray value.
  pub fn gray(&self) -> Option<u8> {
    match self {
      FrameValue::Gray(value) => Some(*value),
      _ => None,
    }
  }
}

impl Display for FrameValue {
  fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
    match self {
      FrameValue::Raw(_) => write!(f, "raw"),
      FrameValue::Invalid(_) => write!(f, "invalid"),
      FrameValue::Float(_) => write!(f, "float"),
      FrameValue::RGBX(_) => write!(f, "RGBX"),
      FrameValue::Gray(_) => write!(f, "gray"),
    }
  }
}
