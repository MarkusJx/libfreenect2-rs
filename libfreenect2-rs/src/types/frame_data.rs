use crate::types::frame::{FrameFormat, Freenect2Frame};
use std::fmt;
use std::fmt::{Display, Formatter};

/// An iterator over the rows in a frame.
/// Each row is an iterator over the values in that row.
/// The values are in the format specified by the frame.
/// The values are in row-major order.
/// The actual values can be accessed by iterating over the
/// returned [`FrameRowIter`] iterator.
pub struct FrameDataIter<'a> {
  data: FrameDataInner<'a>,
  index: usize,
}

impl<'a> FrameDataIter<'a> {
  pub(crate) fn new(frame: &'a dyn Freenect2Frame) -> Self {
    Self {
      data: FrameDataInner {
        data: frame.raw_data(),
        width: frame.width(),
        height: frame.height(),
        bytes_per_pixel: frame.bytes_per_pixel(),
        format: frame.format(),
      },
      index: 0,
    }
  }
}

/// An iterator over the values in a row of a frame.
/// The values are in the format specified by the frame.
pub struct FrameRowIter<'a> {
  data: FrameDataInner<'a>,
  start_index: usize,
  index: usize,
}

#[derive(Clone)]
struct FrameDataInner<'a> {
  data: &'a [u8],
  width: usize,
  height: usize,
  bytes_per_pixel: usize,
  format: FrameFormat,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
/// A RGBX value.
/// The `r`, `g`, and `b` fields are the red, green,
/// and blue values, respectively.
/// The `x` field is unused.
pub struct RGBX {
  /// The red value.
  pub r: u8,
  /// The green value.
  pub g: u8,
  /// The blue value.
  pub b: u8,
  /// Unused.
  pub x: u8,
}

impl RGBX {
  pub fn raw(&self) -> [u8; 4] {
    [self.r, self.g, self.b, self.x]
  }
}

impl<'a> Iterator for FrameDataIter<'a> {
  type Item = FrameRowIter<'a>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.index >= self.data.height {
      return None;
    }

    let index = self.index * self.data.width * self.data.bytes_per_pixel;
    self.index += 1;

    Some(FrameRowIter {
      data: self.data.clone(),
      start_index: index,
      index: 0,
    })
  }
}

impl<'a> Iterator for FrameRowIter<'a> {
  type Item = FrameValue;

  fn next(&mut self) -> Option<Self::Item> {
    if self.index >= self.data.width {
      return None;
    }

    let index = self.index * self.data.bytes_per_pixel + self.start_index;
    self.index += 1;

    Some(match self.data.format {
      FrameFormat::Float => FrameValue::Float(f32::from_ne_bytes(
        self.data.data[index..index + 4].try_into().unwrap(),
      )),
      FrameFormat::Gray => FrameValue::Gray(self.data.data[index]),
      FrameFormat::Raw => {
        FrameValue::Raw(self.data.data[index..index + self.data.bytes_per_pixel].to_vec())
      }
      FrameFormat::Invalid => FrameValue::Invalid(self.data.data[index]),
      FrameFormat::RGBX => FrameValue::RGBX(RGBX {
        r: self.data.data[index],
        g: self.data.data[index + 1],
        b: self.data.data[index + 2],
        x: self.data.data[index + 3],
      }),
      FrameFormat::BGRX => FrameValue::RGBX(RGBX {
        r: self.data.data[index + 2],
        g: self.data.data[index + 1],
        b: self.data.data[index],
        x: self.data.data[index + 3],
      }),
    })
  }
}

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
