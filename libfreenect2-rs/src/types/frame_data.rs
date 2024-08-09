use crate::frame::Freenect2Frame;
use crate::frame_data_iter::FrameDataIter;
use std::fmt::Display;

/// Frame data.
pub enum FrameData<'a> {
  /// Raw data.
  Raw(RawData<'a>),
  /// Gray data.
  Gray(GrayData<'a>),
  /// Float data.
  Float(FloatData<'a>),
  /// RGBX data.
  RGBX(RGBXData<'a>),
  /// Invalid data.
  Invalid,
}

impl<'a> FrameData<'a> {
  /// Expect the value to be raw data.
  ///
  /// # Panics
  /// Panics if the value is not raw data.
  pub fn expect_raw(self) -> RawData<'a> {
    match self {
      FrameData::Raw(value) => value,
      _ => panic!("Expected raw value, found {}", self),
    }
  }

  /// Expect the value to be gray data.
  ///
  /// # Panics
  /// Panics if the value is not gray data.
  pub fn expect_gray(self) -> GrayData<'a> {
    match self {
      FrameData::Gray(value) => value,
      _ => panic!("Expected gray value, found {}", self),
    }
  }

  /// Expect the value to be float data.
  ///
  /// # Panics
  /// Panics if the value is not float data.
  pub fn expect_float(self) -> FloatData<'a> {
    match self {
      FrameData::Float(value) => value,
      _ => panic!("Expected float value, found {}", self),
    }
  }

  /// Expect the value to be RGBX data.
  ///
  /// # Panics
  /// Panics if the value is not RGBX data.
  pub fn expect_rgbx(self) -> RGBXData<'a> {
    match self {
      FrameData::RGBX(value) => value,
      _ => panic!("Expected RGBX value, found {}", self),
    }
  }

  /// Get the raw data, if the value is raw data.
  /// Returns [`None`] if the value is not raw data.
  pub fn raw(self) -> Option<RawData<'a>> {
    match self {
      FrameData::Raw(value) => Some(value),
      _ => None,
    }
  }

  /// Get the raw data, if the value is gray data.
  /// Returns [`None`] if the value is not gray data.
  pub fn gray(self) -> Option<GrayData<'a>> {
    match self {
      FrameData::Gray(value) => Some(value),
      _ => None,
    }
  }

  /// Get the raw data, if the value is float data.
  /// Returns [`None`] if the value is not float data.
  pub fn float(self) -> Option<FloatData<'a>> {
    match self {
      FrameData::Float(value) => Some(value),
      _ => None,
    }
  }

  /// Get the raw data, if the value is RGBX data.
  /// Returns [`None`] if the value is not RGBX data.
  pub fn rgbx(self) -> Option<RGBXData<'a>> {
    match self {
      FrameData::RGBX(value) => Some(value),
      _ => None,
    }
  }

  /// Check if the value is invalid.
  pub fn is_invalid(&self) -> bool {
    matches!(self, FrameData::Invalid)
  }
}

impl Display for FrameData<'_> {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      FrameData::Raw(_) => write!(f, "Raw"),
      FrameData::Gray(_) => write!(f, "Gray"),
      FrameData::Float(_) => write!(f, "Float"),
      FrameData::RGBX(_) => write!(f, "RGBX"),
      FrameData::Invalid => write!(f, "Invalid"),
    }
  }
}

/// Raw data.
pub struct RawData<'a>(&'a dyn Freenect2Frame);

impl<'a> RawData<'a> {
  pub(crate) fn new(frame: &'a dyn Freenect2Frame) -> Self {
    Self(frame)
  }
}

/// Gray data.
pub struct GrayData<'a>(&'a dyn Freenect2Frame);

impl<'a> GrayData<'a> {
  pub(crate) fn new(frame: &'a dyn Freenect2Frame) -> Self {
    Self(frame)
  }
}

/// Float data.
pub struct FloatData<'a>(&'a dyn Freenect2Frame);

impl<'a> FloatData<'a> {
  pub(crate) fn new(frame: &'a dyn Freenect2Frame) -> Self {
    Self(frame)
  }

  /// Get the valid pixel value at the specified position.
  /// The position is specified by the `x` and `y` coordinates.
  /// Returns [`None`] if the pixel is not valid.
  /// A pixel is considered valid if it is not NaN and greater than 0.
  /// The pixel value is in meters.
  ///
  /// # Arguments
  /// * `x` - The x coordinate.
  /// * `y` - The y coordinate.
  ///
  /// # Returns
  /// The valid pixel value at the specified position.
  /// Returns [`None`] if the pixel is not valid.
  pub fn get_valid_pixel(&self, x: usize, y: usize) -> Option<f32> {
    let index = self._get_index(x, y);
    let res = f32::from_ne_bytes(self.0.raw_data()[index..index + 4].try_into().unwrap());

    if res.is_nan() || res <= 0f32 {
      None
    } else {
      Some(res)
    }
  }
}

/// RGBX data.
pub struct RGBXData<'a> {
  frame: &'a dyn Freenect2Frame,
  rgb_type: RGBType,
}

enum RGBType {
  Rgb,
  Bgr,
}

impl<'a> RGBXData<'a> {
  pub(crate) fn rgbx(frame: &'a dyn Freenect2Frame) -> Self {
    Self {
      frame,
      rgb_type: RGBType::Rgb,
    }
  }

  pub(crate) fn bgrx(frame: &'a dyn Freenect2Frame) -> Self {
    Self {
      frame,
      rgb_type: RGBType::Bgr,
    }
  }

  /// Get the red value at the specified position.
  /// The position is specified by the `x` and `y` coordinates.
  ///
  /// # Arguments
  /// * `x` - The x coordinate.
  /// * `y` - The y coordinate.
  ///
  /// # Returns
  /// The red value at the specified position.
  ///
  /// # Panics
  /// Panics if the position is out of bounds.
  pub fn red_at(&self, x: usize, y: usize) -> u8 {
    let index = self._get_index(x, y);
    match self.rgb_type {
      RGBType::Rgb => self.frame.raw_data()[index],
      RGBType::Bgr => self.frame.raw_data()[index + 2],
    }
  }

  /// Get the green value at the specified position.
  /// The position is specified by the `x` and `y` coordinates.
  ///
  /// # Arguments
  /// * `x` - The x coordinate.
  /// * `y` - The y coordinate.
  ///
  /// # Returns
  /// The green value at the specified position.
  ///
  /// # Panics
  /// Panics if the position is out of bounds.
  pub fn green_at(&self, x: usize, y: usize) -> u8 {
    let index = self._get_index(x, y);
    self.frame.raw_data()[index + 1]
  }

  /// Get the blue value at the specified position.
  /// The position is specified by the `x` and `y` coordinates.
  ///
  /// # Arguments
  /// * `x` - The x coordinate.
  /// * `y` - The y coordinate.
  ///
  /// # Returns
  /// The blue value at the specified position.
  ///
  /// # Panics
  /// Panics if the position is out of bounds.
  pub fn blue_at(&self, x: usize, y: usize) -> u8 {
    let index = self._get_index(x, y);
    match self.rgb_type {
      RGBType::Rgb => self.frame.raw_data()[index + 2],
      RGBType::Bgr => self.frame.raw_data()[index],
    }
  }
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

/// A trait for frame data values.
pub trait FrameDataValue<'a> {
  /// The type of the value.
  type Value;

  /// Get the pixel value at the specified position.
  /// The position is specified by the `x` and `y` coordinates.
  ///
  /// # Arguments
  /// * `x` - The x coordinate.
  /// * `y` - The y coordinate.
  ///
  /// # Returns
  /// The pixel value at the specified position.
  ///
  /// # Panics
  /// Panics if the position is out of bounds.
  fn get_pixel(&self, x: usize, y: usize) -> Self::Value;

  /// Returns an iterator over the frame data.
  /// The iterator yields rows of frame data.
  /// Each row is an iterator over the individual pixels.
  /// The pixel data is represented as [`FrameValue`].
  /// The values are in the format specified by the frame.
  ///
  /// # Example
  /// ```
  /// use libfreenect2_rs::frame::{Frame, Freenect2Frame};
  /// use libfreenect2_rs::frame_data::FrameDataValue;
  /// use libfreenect2_rs::frame_value::FrameValue;
  ///
  /// let frame = Frame::depth();
  /// for row in frame.data().expect_float().iter() {
  ///   for value in row {
  ///     println!("Depth value: {}", value);
  ///   }
  /// }
  /// ```
  fn iter(&'a self) -> FrameDataIter<'a, Self>
  where
    Self: Sized,
  {
    FrameDataIter::new(self._frame(), self)
  }

  #[doc(hidden)]
  /// Get the frame that the data belongs to.
  /// Meant for internal use.
  fn _frame(&self) -> &'a dyn Freenect2Frame;

  #[doc(hidden)]
  /// Get the index of the pixel at the specified position.
  /// Meant for internal use.
  fn _get_index(&self, x: usize, y: usize) -> usize {
    (y * self._frame().width() + x) * self._frame().bytes_per_pixel()
  }
}

impl<'a> FrameDataValue<'a> for RawData<'a> {
  type Value = u8;

  fn get_pixel(&self, x: usize, y: usize) -> Self::Value {
    self.0.raw_data()[self._get_index(x, y)]
  }

  fn _frame(&self) -> &'a dyn Freenect2Frame {
    self.0
  }
}

impl<'a> FrameDataValue<'a> for GrayData<'a> {
  type Value = u8;

  fn get_pixel(&self, x: usize, y: usize) -> Self::Value {
    self.0.raw_data()[self._get_index(x, y)]
  }

  fn _frame(&self) -> &'a dyn Freenect2Frame {
    self.0
  }
}

impl<'a> FrameDataValue<'a> for FloatData<'a> {
  type Value = f32;

  fn get_pixel(&self, x: usize, y: usize) -> Self::Value {
    let index = self._get_index(x, y);
    f32::from_ne_bytes(self.0.raw_data()[index..index + 4].try_into().unwrap())
  }

  fn _frame(&self) -> &'a dyn Freenect2Frame {
    self.0
  }
}

impl<'a> FrameDataValue<'a> for RGBXData<'a> {
  type Value = RGBX;

  fn get_pixel(&self, x: usize, y: usize) -> Self::Value {
    let index = self._get_index(x, y);
    match self.rgb_type {
      RGBType::Rgb => RGBX {
        r: self.frame.raw_data()[index],
        g: self.frame.raw_data()[index + 1],
        b: self.frame.raw_data()[index + 2],
        x: self.frame.raw_data()[index + 3],
      },
      RGBType::Bgr => RGBX {
        b: self.frame.raw_data()[index],
        g: self.frame.raw_data()[index + 1],
        r: self.frame.raw_data()[index + 2],
        x: self.frame.raw_data()[index + 3],
      },
    }
  }

  fn _frame(&self) -> &'a dyn Freenect2Frame {
    self.frame
  }
}
