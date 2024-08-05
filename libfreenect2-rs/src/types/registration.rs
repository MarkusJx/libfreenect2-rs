use crate::ffi::libfreenect2;
use crate::frame::{AsFrame, Frame, FrameFormat, Freenect2Frame};
use cxx::UniquePtr;

macro_rules! ensure_frame {
  ($val: ident, $($format: ident)|+, $width: expr, $height: expr) => {
    anyhow::ensure!(
      ($($val.format() == FrameFormat::$format) ||+)
        && $val.width() == $width
        && $val.height() == $height
        && $val.bytes_per_pixel() == 4
        && $val.raw_data().len() == $val.raw_data_len(),
      format!(
        "{} {:?}, width: {}, height: {}, bytes per pixel: {}",
        concat!(
          "Invalid ",
          stringify!($val),
          " frame. Expected format: ",
          stringify!($($format) or+),
          ", width: ",
          $width,
          ", height: ",
          $height,
          ", bytes per pixel: 4. Got format:"
        ),
        $val.format(),
        $val.width(),
        $val.height(),
        $val.bytes_per_pixel(),
      )
    )
  };
}

pub struct Registration(UniquePtr<libfreenect2::Registration>);

impl Registration {
  pub(crate) fn new(inner: UniquePtr<libfreenect2::Registration>) -> Self {
    Self(inner)
  }

  pub fn map_depth_to_color<'a, 'b, F1: AsFrame<'a>, F2: AsFrame<'b>>(
    &self,
    depth: &'a F1,
    color: &'b F2,
    undistorted_depth: &mut Frame,
    color_depth_image: &mut Frame,
    enable_filter: bool,
  ) -> anyhow::Result<()> {
    let depth = depth.as_frame();
    let color = color.as_frame();

    ensure_frame!(depth, Float, 512, 424);
    ensure_frame!(color, RGBX | BGRX, 1920, 1080);
    ensure_frame!(undistorted_depth, Float, 512, 424);
    ensure_frame!(color_depth_image, RGBX | BGRX, 512, 424);

    unsafe {
      self
        .0
        .map_depth_to_color(
          &depth.inner,
          &color.inner,
          undistorted_depth.inner.pin_mut(),
          color_depth_image.inner.pin_mut(),
          enable_filter,
        )
        .map_err(Into::into)
    }
  }

  pub fn map_depth_to_full_color<'a, 'b, F1: AsFrame<'a>, F2: AsFrame<'b>>(
    &self,
    depth: &'a F1,
    color: &'b F2,
    undistorted_depth: &mut Frame,
    color_depth_image: &mut Frame,
    enable_filter: bool,
    big_depth: &mut Frame,
  ) -> anyhow::Result<()> {
    let depth = depth.as_frame();
    let color = color.as_frame();

    ensure_frame!(depth, Float, 512, 424);
    ensure_frame!(color, RGBX | BGRX, 1920, 1080);
    ensure_frame!(undistorted_depth, Float, 512, 424);
    ensure_frame!(color_depth_image, RGBX | BGRX, 512, 424);
    ensure_frame!(big_depth, Float, 1920, 1082);

    unsafe {
      self
        .0
        .map_depth_to_full_color(
          &depth.inner,
          &color.inner,
          undistorted_depth.inner.pin_mut(),
          color_depth_image.inner.pin_mut(),
          enable_filter,
          big_depth.inner.pin_mut(),
        )
        .map_err(Into::into)
    }
  }

  pub fn undistort_depth<'a, F: AsFrame<'a>>(
    &self,
    depth: &'a F,
    undistorted_depth: &mut Frame,
  ) -> anyhow::Result<()> {
    let depth = depth.as_frame();

    ensure_frame!(depth, Float, 512, 424);
    ensure_frame!(undistorted_depth, Float, 512, 424);

    unsafe {
      self
        .0
        .undistort_depth(&depth.inner, undistorted_depth.inner.pin_mut())
        .map_err(Into::into)
    }
  }
}

unsafe impl Send for Registration {}
unsafe impl Sync for Registration {}
