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

/// A registration object that can be used to map depth frames to color frames.
/// Can be created by calling [`crate::freenect2_device::Freenect2Device::get_registration`].
pub struct Registration(UniquePtr<libfreenect2::Registration>);

impl Registration {
  pub(crate) fn new(inner: UniquePtr<libfreenect2::Registration>) -> Self {
    Self(inner)
  }

  /// Maps a depth frame to a color frame.
  /// The resulting frames are stored in `undistorted_depth` and `color_depth_image`.
  /// If `enable_filter` is true, pixels not visible will be filtered out.
  ///
  /// # Arguments
  /// * `depth` - The depth frame to map.
  ///    Must be of format [`FrameFormat::Float`] and have a resolution of 512x424.
  /// * `color` - The color frame to map to.
  ///    Must be of format [`FrameFormat::RGBX`] or [`FrameFormat::BGRX`] and have a resolution of 1920x1080.
  /// * `undistorted_depth` - The resulting undistorted depth frame.
  ///    Must be of format [`FrameFormat::Float`] and have a resolution of 512x424.
  ///    Can be created using [`Frame::depth`].
  /// * `color_depth_image` - The resulting color depth image frame.
  ///    Must be of format [`FrameFormat::RGBX`] or [`FrameFormat::BGRX`] and have a resolution of 512x424.
  ///    The exact format depends on the format of the `color` frame and will be automatically set.
  ///    Can be created using [`Frame::color_for_depth`].
  /// * `enable_filter` - Whether to filter out pixels not visible to both cameras.
  ///
  /// # Errors
  /// Returns an error if the frames have invalid formats or resolutions.
  ///
  /// # Example
  /// ```ignored
  /// use std::collections::HashSet;
  /// use libfreenect2_rs::frame::{Frame, FrameFormat};
  /// use libfreenect2_rs::frame_listener::OwnedFramesMultiFrameListener;
  /// use libfreenect2_rs::frame_type::FrameType;
  /// use libfreenect2_rs::freenect2::Freenect2;
  ///
  /// let mut freenect2 = Freenect2::new().unwrap();
  /// let mut device = freenect2.open_default_device().unwrap();
  ///
  /// let frame_listener = OwnedFramesMultiFrameListener::new(&[FrameType::Color, FrameType::Depth]).unwrap();
  ///
  /// device.set_color_frame_listener(&frame_listener).unwrap();
  /// device.set_ir_and_depth_frame_listener(&frame_listener).unwrap();
  ///
  /// device.start().unwrap();
  /// let registration = device.get_registration().unwrap();
  ///
  /// let frames = frame_listener.get_frames().unwrap();
  ///
  /// let color = frames.expect_color().unwrap();
  /// let depth = frames.expect_depth().unwrap();
  ///
  /// let mut undistorted_depth = Frame::depth();
  /// let mut color_image = Frame::color_for_depth();
  ///
  /// registration.map_depth_to_color(
  ///   depth,
  ///   color,
  ///   &mut undistorted_depth,
  ///   &mut color_image,
  ///   true
  /// ).unwrap();
  /// ```
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

  /// Map a depth frame onto a color frame.
  /// The resulting depth frame will have the same resolution
  /// as the color frame plus one blank line at the top and bottom (1920x1082).
  ///
  /// # Arguments
  /// * `depth` - The depth frame to map.
  ///    Must be of format [`FrameFormat::Float`] and have a resolution of 512x424.
  /// * `color` - The color frame to map to.
  ///    Must be of format [`FrameFormat::RGBX`] or [`FrameFormat::BGRX`] and have a resolution of 1920x1080.
  /// * `undistorted_depth` - The resulting undistorted depth frame.
  ///    Must be of format [`FrameFormat::Float`] and have a resolution of 512x424.
  ///    Can be created using [`Frame::depth`].
  /// * `color_depth_image` - The resulting color depth image frame.
  ///    Must be of format [`FrameFormat::RGBX`] or [`FrameFormat::BGRX`] and have a resolution of 512x424.
  ///    The exact format depends on the format of the `color` frame and will be automatically set.
  ///    Can be created using [`Frame::color_for_depth`].
  /// * `enable_filter` - Whether to filter out pixels not visible to both cameras.
  /// * `big_depth` - The scaled up depth frame.
  ///    Must be of format [`FrameFormat::Float`] and have a resolution of 1920x1082.
  ///    Can be created using [`Frame::depth_full_color`].
  ///
  /// # Errors
  /// Returns an error if the frames have invalid formats or resolutions.
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

  /// Un-distort a depth frame.
  ///
  /// # Arguments
  /// * `depth` - The depth frame to map.
  ///    Must be of format [`FrameFormat::Float`] and have a resolution of 512x424.
  /// * `undistorted_depth` - The resulting undistorted depth frame.
  ///    Must be of format [`FrameFormat::Float`] and have a resolution of 512x424.
  ///    Can be created using [`Frame::depth`].
  ///
  /// # Errors
  /// Returns an error if the frames have invalid formats or resolutions.
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
