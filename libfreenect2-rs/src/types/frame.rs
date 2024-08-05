use std::ops::Deref;
use std::time::Duration;

use crate::ffi;
use crate::frame_data::{FrameValue, RGBX};
use crate::types::frame_data::FrameDataIter;

pub enum FrameReference<'a> {
  Owned(Frame<'a>),
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

pub trait AsFrame<'a> {
  fn as_frame(&'a self) -> FrameReference<'a>;
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
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

  fn sequence(&self) -> u32;

  fn exposure(&self) -> f32;

  fn gain(&self) -> f32;

  fn gamma(&self) -> f32;

  fn status(&self) -> u32;

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
