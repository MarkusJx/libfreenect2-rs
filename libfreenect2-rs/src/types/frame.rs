use std::time::Duration;

use crate::ffi;
use crate::types::frame_data::FrameDataIter;

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum FrameFormat {
  Invalid,
  Raw,
  Float,
  RGBX,
  BGRX,
  Gray,
}

#[cfg(feature = "image")]
pub enum FrameImage {
  RGB(image::RgbImage),
  Gray(image::GrayImage),
  Float(image::ImageBuffer<image::Luma<f32>, Vec<f32>>),
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

pub trait Freenect2Frame {
  fn width(&self) -> usize;

  fn height(&self) -> usize;

  fn bytes_per_pixel(&self) -> usize;

  fn timestamp(&self) -> u32;

  fn timestamp_as_duration(&self) -> Duration {
    let timestamp = self.timestamp() as f64 * 0.125f64;
    Duration::from_millis(timestamp as u64)
  }

  fn raw_data(&self) -> &[u8];

  fn raw_data_len(&self) -> usize {
    self.width() * self.height() * self.bytes_per_pixel()
  }

  fn format(&self) -> FrameFormat;

  fn iter(&self) -> FrameDataIter
  where
    Self: Sized,
  {
    FrameDataIter::new(self)
  }

  #[cfg(feature = "image")]
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
}

pub struct Frame<'a> {
  pub(crate) inner: cxx::UniquePtr<ffi::libfreenect2::Frame<'a>>,
}

impl<'a> Frame<'a> {
  pub(crate) fn new(inner: cxx::UniquePtr<ffi::libfreenect2::Frame<'a>>) -> Self {
    Self { inner }
  }

  pub fn to_owned(&self) -> OwnedFrame {
    OwnedFrame::new(
      self.width(),
      self.height(),
      self.bytes_per_pixel(),
      self.timestamp(),
      self.raw_data().to_vec(),
      self.format(),
    )
  }
}

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

  fn format(&self) -> FrameFormat {
    unsafe { self.inner.format() }.into()
  }
}

pub struct OwnedFrame {
  width: usize,
  height: usize,
  bytes_per_pixel: usize,
  timestamp: u32,
  data: Vec<u8>,
  format: FrameFormat,
}

impl OwnedFrame {
  fn new(
    width: usize,
    height: usize,
    bytes_per_pixel: usize,
    timestamp: u32,
    data: Vec<u8>,
    format: FrameFormat,
  ) -> Self {
    Self {
      width,
      height,
      bytes_per_pixel,
      timestamp,
      data,
      format,
    }
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

  fn format(&self) -> FrameFormat {
    self.format
  }
}
