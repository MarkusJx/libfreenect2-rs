use std::ops::Deref;
use std::time::Duration;

use crate::ffi;
use crate::frame_data::{FrameValue, RGBX};
use crate::types::frame_data::FrameDataIter;

/// A [`Frame`] or a reference to a [`Frame`].
pub enum FrameReference<'a> {
  /// An owned [`Frame`].
  Owned(Frame<'a>),
  /// A borrowed [`Frame`].
  Borrowed(&'a Frame<'a>),
}

impl<'a> Deref for FrameReference<'a> {
  type Target = Frame<'a>;

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
pub trait AsFrame<'a> {
  /// Returns a reference to a [`Frame`].
  fn as_frame(&'a self) -> FrameReference<'a>;
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

  /// Returns an iterator over the frame data.
  /// The iterator yields rows of frame data.
  /// Each row is an iterator over the individual pixels.
  /// The pixel data is represented as [`FrameValue`].
  /// The values are in the format specified by the frame.
  ///
  /// # Example
  /// ```
  /// use libfreenect2_rs::frame::{Frame, Freenect2Frame};
  /// use libfreenect2_rs::frame_data::FrameValue;
  ///
  /// let frame = Frame::depth();
  /// for row in frame.iter() {
  ///   for value in row {
  ///     match value {
  ///       FrameValue::Float(value) => {
  ///         println!("Depth value: {}", value);
  ///       },
  ///       _ => unreachable!(),
  ///     }
  ///   }
  /// }
  /// ```
  fn iter(&self) -> FrameDataIter
  where
    Self: Sized,
  {
    FrameDataIter::new(self)
  }

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
        let Some(data) = self
          .iter()
          .flat_map(|row| row.map(|value| value.rgbx().map(|r| r.raw()[..3].to_vec())))
          .collect::<Option<Vec<_>>>()
          .map(|data| data.into_iter().flatten().collect::<Vec<_>>())
        else {
          return FrameImage::Invalid;
        };

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
        let Some(data) = self
          .iter()
          .flat_map(|row| row.map(|value| value.float()))
          .collect::<Option<Vec<_>>>()
        else {
          return FrameImage::Invalid;
        };

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
  /// # Panics
  /// Panics if the coordinates are outside the frame dimensions,
  /// i.e. `x >=` [`Self::width`] or `y >=` [`Self::height`].
  ///
  /// # Example
  /// ```
  /// use libfreenect2_rs::frame::{Frame, Freenect2Frame};
  /// use libfreenect2_rs::frame_data::FrameValue;
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
    match self.format() {
      FrameFormat::RGBX => {
        let data = self.raw_data();
        FrameValue::RGBX(RGBX {
          r: data[index],
          g: data[index + 1],
          b: data[index + 2],
          x: data[index + 3],
        })
      }
      FrameFormat::BGRX => {
        let data = self.raw_data();
        FrameValue::RGBX(RGBX {
          r: data[index + 2],
          g: data[index + 1],
          b: data[index],
          x: data[index + 3],
        })
      }
      FrameFormat::Gray => {
        let data = self.raw_data();
        FrameValue::Gray(data[index])
      }
      FrameFormat::Float => {
        let data = self.raw_data();
        FrameValue::Float(f32::from_ne_bytes(
          data[index..index + 4].try_into().unwrap(),
        ))
      }
      FrameFormat::Raw => {
        FrameValue::Raw(self.raw_data()[index..index + self.bytes_per_pixel()].to_vec())
      }
      FrameFormat::Invalid => FrameValue::Invalid(self.raw_data()[index]),
    }
  }
}

pub struct Frame<'a> {
  pub(crate) inner: cxx::UniquePtr<ffi::libfreenect2::Frame<'a>>,
}

impl<'a> Frame<'a> {
  pub(crate) fn new(inner: cxx::UniquePtr<ffi::libfreenect2::Frame<'a>>) -> Self {
    Self { inner }
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

impl<'a> AsFrame<'a> for Frame<'a> {
  fn as_frame(&'a self) -> FrameReference<'a> {
    FrameReference::Borrowed(self)
  }
}

unsafe impl Send for Frame<'_> {}
unsafe impl Sync for Frame<'_> {}

impl Freenect2Frame for Frame<'_> {
  fn width(&self) -> usize {
    unsafe { self.inner.width() as usize }
  }

  fn height(&self) -> usize {
    unsafe { self.inner.height() as usize }
  }

  fn bytes_per_pixel(&self) -> usize {
    unsafe { self.inner.bytes_per_pixel() as usize }
  }

  fn timestamp(&self) -> u32 {
    unsafe { self.inner.timestamp() }
  }

  fn raw_data(&self) -> &[u8] {
    unsafe { std::slice::from_raw_parts(self.inner.data(), self.raw_data_len()) }
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

impl<'a> AsFrame<'a> for OwnedFrame {
  fn as_frame(&'a self) -> FrameReference<'a> {
    FrameReference::Owned(self.to_frame())
  }
}

impl From<Frame<'_>> for OwnedFrame {
  fn from(frame: Frame) -> Self {
    frame.to_owned()
  }
}
