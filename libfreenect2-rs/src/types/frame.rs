use std::ops::Deref;
use std::time::Duration;

use crate::ffi;
#[cfg(feature = "image")]
use crate::frame_data::FrameDataValue;
use crate::frame_data::{FloatData, FrameData, GrayData, RGBXData, RawData, RGBX};
use crate::frame_value::FrameValue;

/// A [`Frame`] or a reference to a [`Frame`].
pub enum FrameReference<'a, 'b: 'a> {
  /// An owned [`Frame`].
  Owned(Frame<'b>),
  /// A borrowed [`Frame`].
  Borrowed(&'a Frame<'b>),
}

impl<'a, 'b: 'a> Deref for FrameReference<'a, 'b> {
  type Target = Frame<'b>;

  fn deref(&self) -> &Self::Target {
    match self {
      FrameReference::Owned(frame) => frame,
      FrameReference::Borrowed(frame) => frame,
    }
  }
}

/// A trait for types that can be converted to
/// a reference to a [`Frame`] or are a [`Frame`].
/// Currently implemented by [`Frame`] and [`OwnedFrame`].
pub trait AsFrame<'a, 'b: 'a> {
  /// Returns a reference to a [`Frame`].
  fn as_frame(&'a self) -> FrameReference<'a, 'b>;
}

/// Frame format
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum FrameFormat {
  /// Invalid format
  Invalid,
  /// Raw bitstream. 'bytes_per_pixel' defines the number of bytes
  Raw,
  /// A 4-byte float per pixel
  Float,
  /// 4 bytes of R, G, B, and unused per pixel
  RGBX,
  /// 4 bytes of B, G, R, and unused per pixel
  BGRX,
  /// 1 byte of gray per pixel
  Gray,
}

#[cfg(feature = "image")]
/// A [`Frame`] converted to an [`image`] type.
pub enum FrameImage {
  /// An RGB image. The source for the image may
  /// be either [`FrameFormat::RGBX`] or [`FrameFormat::BGRX`].
  RGB(image::RgbImage),
  /// A grayscale image.
  Gray(image::GrayImage),
  /// A floating-point image. Usually used for depth and IR data.
  Float(image::ImageBuffer<image::Luma<f32>, Vec<f32>>),
  /// An invalid image.
  Invalid,
}

impl From<ffi::libfreenect2::FrameFormat> for FrameFormat {
  fn from(value: ffi::libfreenect2::FrameFormat) -> Self {
    match value {
      ffi::libfreenect2::FrameFormat::Raw => FrameFormat::Raw,
      ffi::libfreenect2::FrameFormat::Float => FrameFormat::Float,
      ffi::libfreenect2::FrameFormat::RGBX => FrameFormat::RGBX,
      ffi::libfreenect2::FrameFormat::BGRX => FrameFormat::BGRX,
      ffi::libfreenect2::FrameFormat::Gray => FrameFormat::Gray,
      _ => FrameFormat::Invalid,
    }
  }
}

impl From<FrameFormat> for ffi::libfreenect2::FrameFormat {
  fn from(value: FrameFormat) -> Self {
    match value {
      FrameFormat::Raw => ffi::libfreenect2::FrameFormat::Raw,
      FrameFormat::Float => ffi::libfreenect2::FrameFormat::Float,
      FrameFormat::RGBX => ffi::libfreenect2::FrameFormat::RGBX,
      FrameFormat::BGRX => ffi::libfreenect2::FrameFormat::BGRX,
      FrameFormat::Gray => ffi::libfreenect2::FrameFormat::Gray,
      FrameFormat::Invalid => ffi::libfreenect2::FrameFormat::Invalid,
    }
  }
}

/// A trait for frame types.
/// This trait is implemented for both [`Frame`] and [`OwnedFrame`].
pub trait Freenect2Frame {
  /// Returns the width of the frame in pixels.
  fn width(&self) -> usize;

  /// Returns the height of the frame in pixels.
  fn height(&self) -> usize;

  /// Returns the number of bytes per pixel.
  fn bytes_per_pixel(&self) -> usize;

  /// Returns the timestamp of the frame.
  /// The timestamp is in 1/8th of a millisecond.
  /// To convert it to a [`Duration`], multiply it by 0.125.
  /// Usually incrementing by 266 (30Hz) or 533 (15Hz).
  fn timestamp(&self) -> u32;

  /// Returns the timestamp of the frame as a [`Duration`].
  fn timestamp_as_duration(&self) -> Duration {
    let timestamp = self.timestamp() as f64 * 0.125f64;
    Duration::from_millis(timestamp as u64)
  }

  /// Returns the raw data of the frame.
  /// The data is stored in row-major order.
  /// The length of the data must be equal to `width * height * bytes_per_pixel`.
  /// The length can also be retrieved using [`Self::raw_data_len`].
  fn raw_data(&self) -> &[u8];

  /// Returns the length of the raw data.
  /// The length is equal to `width * height * bytes_per_pixel`.
  fn raw_data_len(&self) -> usize {
    self.width() * self.height() * self.bytes_per_pixel()
  }

  /// Increasing sequence number
  fn sequence(&self) -> u32;

  /// From 0.5 (very bright) to ~60.0 (fully covered)
  fn exposure(&self) -> f32;

  /// From 1.0 (bright) to 1.5 (covered)
  fn gain(&self) -> f32;

  /// From 1.0 (bright) to 6.4 (covered)
  fn gamma(&self) -> f32;

  /// zero if ok; non-zero for errors.
  fn status(&self) -> u32;

  fn format(&self) -> FrameFormat;

  #[cfg(feature = "image")]
  /// Converts the frame to an [`image`] type.
  /// The image type is determined by the frame format.
  /// The conversion is only available when the `image` feature is enabled.
  /// If the frame format is not supported, [`FrameImage::Invalid`] is returned.
  ///
  /// # Example
  /// ```
  /// use libfreenect2_rs::frame::{Frame, Freenect2Frame};
  /// use libfreenect2_rs::frame::FrameImage;
  ///
  /// let frame = Frame::color_for_depth();
  /// let image = frame.as_image();
  ///
  /// match image {
  ///   FrameImage::RGB(rgb) => {
  ///     let pixel = rgb.get_pixel(0, 0);
  ///     // Do something with the pixel.
  ///     // The pixel in this example will contain random values
  ///     // because the frame is not initialized.
  ///   },
  ///   _ => unreachable!(),
  /// }
  /// ```
  fn as_image(&self) -> FrameImage
  where
    Self: Sized,
  {
    match self.format() {
      FrameFormat::BGRX | FrameFormat::RGBX => {
        let data = self
          .data()
          .expect_rgbx()
          .iter()
          .flat_map(|row| row.flat_map(|value| value.raw()[..3].to_vec()))
          .collect::<Vec<_>>();

        image::RgbImage::from_raw(self.width() as _, self.height() as _, data)
          .map_or(FrameImage::Invalid, FrameImage::RGB)
      }
      FrameFormat::Gray => image::GrayImage::from_raw(
        self.width() as _,
        self.height() as _,
        self.raw_data().to_vec(),
      )
      .map_or(FrameImage::Invalid, FrameImage::Gray),
      FrameFormat::Float => {
        let data = self
          .data()
          .expect_float()
          .iter()
          .flatten()
          .collect::<Vec<_>>();

        image::ImageBuffer::from_raw(self.width() as _, self.height() as _, data)
          .map_or(FrameImage::Invalid, FrameImage::Float)
      }
      FrameFormat::Invalid | FrameFormat::Raw => FrameImage::Invalid,
    }
  }

  /// Returns the pixel at the specified coordinates.
  /// The coordinates must be within the frame dimensions.
  /// The pixel data is represented as [`FrameValue`].
  /// The values are in the format specified by the frame.
  ///
  /// For improved performance, consider using [`Self::data`] and
  /// [`FrameDataValue::get_pixel`] instead:
  /// ```
  /// use libfreenect2_rs::frame::{Frame, Freenect2Frame};
  /// use libfreenect2_rs::frame_data::FrameDataValue;
  ///
  /// // Get the frame
  /// let frame = Frame::depth();
  ///
  /// // Get the frame data
  /// let data = frame.data().expect_float();
  /// let pixel = data.get_pixel(0, 0);
  /// ```
  ///
  /// # Panics
  /// Panics if the coordinates are outside the frame dimensions,
  /// i.e. `x >=` [`Self::width`] or `y >=` [`Self::height`].
  ///
  /// # Example
  /// ```
  /// use libfreenect2_rs::frame::{Frame, Freenect2Frame};
  /// use libfreenect2_rs::frame_value::FrameValue;
  ///
  /// let frame = Frame::depth();
  /// let pixel = frame.get_pixel(0, 0);
  ///
  /// match pixel {
  ///   FrameValue::Float(value) => {
  ///     println!("Depth value: {}", value);
  ///   }
  ///   _ => unreachable!(),
  /// }
  /// ```
  fn get_pixel(&self, x: usize, y: usize) -> FrameValue {
    assert!(x < self.width(), "x: {} >= width: {}", x, self.width());
    assert!(y < self.height(), "y: {} >= height: {}", y, self.height());

    let index = (y * self.width() + x) * self.bytes_per_pixel();
    let data = self.raw_data();
    match self.format() {
      FrameFormat::RGBX => FrameValue::RGBX(RGBX {
        r: data[index],
        g: data[index + 1],
        b: data[index + 2],
        x: data[index + 3],
      }),
      FrameFormat::BGRX => FrameValue::RGBX(RGBX {
        r: data[index + 2],
        g: data[index + 1],
        b: data[index],
        x: data[index + 3],
      }),
      FrameFormat::Gray => FrameValue::Gray(data[index]),
      FrameFormat::Float => FrameValue::Float(f32::from_ne_bytes(
        data[index..index + 4].try_into().unwrap(),
      )),
      FrameFormat::Raw => FrameValue::Raw(data[index..index + self.bytes_per_pixel()].to_vec()),
      FrameFormat::Invalid => FrameValue::Invalid(self.raw_data()[index]),
    }
  }

  /// Returns the data of the frame.
  /// The data is represented as [`FrameData`].
  /// The data is in the format specified by the frame.
  /// The data can be used to iterate over the frame data.
  /// The data can also be used to get the pixel at a specific position.
  ///
  /// # Example
  /// ```
  /// use libfreenect2_rs::frame::{Frame, Freenect2Frame};
  /// use libfreenect2_rs::frame_data::FrameDataValue;
  ///
  /// let frame = Frame::depth();
  /// let data = frame.data();
  ///
  /// for row in data.expect_float().iter() {
  ///   for value in row {
  ///     println!("Value: {}", value);
  ///   }
  /// }
  /// ```
  fn data(&self) -> FrameData
  where
    Self: Sized,
  {
    match self.format() {
      FrameFormat::RGBX => FrameData::RGBX(RGBXData::rgbx(self)),
      FrameFormat::BGRX => FrameData::RGBX(RGBXData::bgrx(self)),
      FrameFormat::Gray => FrameData::Gray(GrayData::new(self)),
      FrameFormat::Float => FrameData::Float(FloatData::new(self)),
      FrameFormat::Raw => FrameData::Raw(RawData::new(self)),
      FrameFormat::Invalid => FrameData::Invalid,
    }
  }
}

/// A native libfreenect2 frame.
/// Contains an owned pointer to a libfreenect2 frame.
/// Can't be cloned or copied. Use [`OwnedFrame`] for that,
/// which can be created using [`Frame::to_owned`].
pub struct Frame<'a> {
  pub(crate) inner: cxx::UniquePtr<ffi::libfreenect2::Frame<'a>>,
  width: usize,
  height: usize,
  bytes_per_pixel: usize,
  raw_data: &'a [u8],
}

impl<'a> Frame<'a> {
  pub(crate) fn new(inner: cxx::UniquePtr<ffi::libfreenect2::Frame<'a>>) -> Self {
    let width = unsafe { inner.width() as usize };
    let height = unsafe { inner.height() as usize };
    let bytes_per_pixel = unsafe { inner.bytes_per_pixel() as usize };

    Self {
      width,
      height,
      bytes_per_pixel,
      raw_data: unsafe {
        std::slice::from_raw_parts(inner.data(), width * height * bytes_per_pixel)
      },
      inner,
    }
  }

  /// Create a new depth frame.
  /// The frame has a resolution of 512x424 pixels.
  /// The frame format is [`FrameFormat::Float`].
  ///
  /// Usually used as third parameter for
  /// [`crate::registration::Registration::map_depth_to_color`] or
  /// [`crate::registration::Registration::map_depth_to_full_color`]
  /// or as second parameter for
  /// [`crate::registration::Registration::undistort_depth`].
  pub fn depth() -> Self {
    Self::new(unsafe {
      ffi::libfreenect2::create_frame(
        512,
        424,
        4,
        std::ptr::null_mut(),
        0,
        0,
        0.0,
        0.0,
        0.0,
        0,
        ffi::libfreenect2::FrameFormat::Float,
      )
    })
  }

  /// Create a new color frame.
  /// The frame has a resolution of 1920x1080 pixels.
  /// The frame format is [`FrameFormat::RGBX`].
  ///
  /// The frame is usually used as the fourth parameter for
  /// [`crate::registration::Registration::map_depth_to_color`] or
  /// [`crate::registration::Registration::map_depth_to_full_color`].
  pub fn color_for_depth() -> Self {
    Self::new(unsafe {
      ffi::libfreenect2::create_frame(
        512,
        424,
        4,
        std::ptr::null_mut(),
        0,
        0,
        0.0,
        0.0,
        0.0,
        0,
        ffi::libfreenect2::FrameFormat::RGBX,
      )
    })
  }

  /// Create a depth frame to match the resolution of a color frame.
  /// The frame format is [`FrameFormat::Float`].
  /// The frame resolution is 1920x1082 pixels.
  /// The frame has two blank lines, one at the top and one at the bottom.
  /// The frame is usually used as the sixth parameter for
  /// [`crate::registration::Registration::map_depth_to_full_color`].
  pub fn depth_full_color() -> Self {
    Self::new(unsafe {
      ffi::libfreenect2::create_frame(
        1920,
        1082,
        4,
        std::ptr::null_mut(),
        0,
        0,
        0.0,
        0.0,
        0.0,
        0,
        ffi::libfreenect2::FrameFormat::Float,
      )
    })
  }

  /// Convert the frame to an owned frame.
  /// The owned frame has the same data as the original frame.
  /// Copies the data of the frame.
  pub fn to_owned(&self) -> OwnedFrame {
    OwnedFrame {
      width: self.width(),
      height: self.height(),
      bytes_per_pixel: self.bytes_per_pixel(),
      timestamp: self.timestamp(),
      data: self.raw_data().to_vec(),
      sequence: self.sequence(),
      exposure: self.exposure(),
      gain: self.gain(),
      gamma: self.gamma(),
      status: self.status(),
      format: self.format(),
    }
  }
}

impl<'a, 'b: 'a> AsFrame<'a, 'b> for Frame<'b> {
  fn as_frame(&'a self) -> FrameReference<'a, 'b> {
    FrameReference::Borrowed(self)
  }
}

unsafe impl Send for Frame<'_> {}
unsafe impl Sync for Frame<'_> {}

impl Freenect2Frame for Frame<'_> {
  fn width(&self) -> usize {
    self.width
  }

  fn height(&self) -> usize {
    self.height
  }

  fn bytes_per_pixel(&self) -> usize {
    self.bytes_per_pixel
  }

  fn timestamp(&self) -> u32 {
    unsafe { self.inner.timestamp() }
  }

  fn raw_data(&self) -> &[u8] {
    self.raw_data
  }

  fn sequence(&self) -> u32 {
    unsafe { self.inner.sequence() }
  }

  fn exposure(&self) -> f32 {
    unsafe { self.inner.exposure() }
  }

  fn gain(&self) -> f32 {
    unsafe { self.inner.gain() }
  }

  fn gamma(&self) -> f32 {
    unsafe { self.inner.gamma() }
  }

  fn status(&self) -> u32 {
    unsafe { self.inner.status() }
  }

  fn format(&self) -> FrameFormat {
    unsafe { self.inner.format() }.into()
  }
}

/// An owned frame.
/// The owned frame is a cloneable version of a [`Frame`].
/// The owned frame can be converted back to a [`Frame`].
#[derive(Clone)]
pub struct OwnedFrame {
  width: usize,
  height: usize,
  bytes_per_pixel: usize,
  timestamp: u32,
  data: Vec<u8>,
  sequence: u32,
  exposure: f32,
  gain: f32,
  gamma: f32,
  status: u32,
  format: FrameFormat,
}

impl OwnedFrame {
  /// Convert the owned frame to a [`Frame`].
  /// The frame has the same data as the owned frame.
  /// The frame is a borrowed reference to the owned frame.
  /// The frame is not a copy of the owned frame.
  /// The frame is only valid as long as the owned frame is valid.
  ///
  /// # Example
  /// ```
  /// use libfreenect2_rs::frame::Frame;
  ///
  /// let frame = Frame::depth();
  /// let owned = frame.to_owned();
  /// let copied_frame = owned.to_frame();
  /// // Success! Copied a lot of data whilst achieving nothing.
  /// ```
  pub fn to_frame(&self) -> Frame {
    Frame::new(unsafe {
      ffi::libfreenect2::create_frame(
        self.width as _,
        self.height as _,
        self.bytes_per_pixel as _,
        self.data.as_ptr() as _,
        self.timestamp,
        self.sequence,
        self.exposure,
        self.gain,
        self.gamma,
        self.status,
        self.format.into(),
      )
    })
  }
}

impl Freenect2Frame for OwnedFrame {
  fn width(&self) -> usize {
    self.width
  }

  fn height(&self) -> usize {
    self.height
  }

  fn bytes_per_pixel(&self) -> usize {
    self.bytes_per_pixel
  }

  fn timestamp(&self) -> u32 {
    self.timestamp
  }

  fn raw_data(&self) -> &[u8] {
    &self.data
  }

  fn sequence(&self) -> u32 {
    self.sequence
  }

  fn exposure(&self) -> f32 {
    self.exposure
  }

  fn gain(&self) -> f32 {
    self.gain
  }

  fn gamma(&self) -> f32 {
    self.gamma
  }

  fn status(&self) -> u32 {
    self.status
  }

  fn format(&self) -> FrameFormat {
    self.format
  }
}

impl<'a> AsFrame<'a, 'a> for OwnedFrame {
  fn as_frame(&'a self) -> FrameReference<'a, 'a> {
    FrameReference::Owned(self.to_frame())
  }
}

impl From<Frame<'_>> for OwnedFrame {
  fn from(frame: Frame) -> Self {
    frame.to_owned()
  }
}
