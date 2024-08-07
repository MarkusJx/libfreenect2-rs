use crate::constants::MAX_DEPTH;
use crate::RenderType;
use anyhow::anyhow;
use libfreenect2_rs::config::Config;
use libfreenect2_rs::frame::{Frame, OwnedFrame};
use libfreenect2_rs::frame_listener::{
  FrameMap, MultiFrameListener, OwnedFramesMultiFrameListener,
};
use libfreenect2_rs::frame_type::FrameType;
use libfreenect2_rs::freenect2::Freenect2;
use libfreenect2_rs::freenect2_device::Freenect2Device;
use libfreenect2_rs::registration::Registration;
use once_cell::sync::OnceCell;

static FRAME_LISTENER: OnceCell<OwnedFramesMultiFrameListener> = OnceCell::new();
static mut FREENECT: Option<Freenect2> = None;
static CONFIG: OnceCell<Config> = OnceCell::new();

pub struct FreenectState<'a> {
  _device: Freenect2Device<'a>,
  registration: Registration,
  render_type: RenderType,
}

impl FreenectState<'_> {
  pub fn new(render_type: RenderType) -> anyhow::Result<Self> {
    let mut device = unsafe {
      FREENECT = Some(Freenect2::new()?);
      FREENECT.as_mut().unwrap().open_default_device()?
    };

    let frame_types: &[FrameType] = if render_type.is_color() {
      &[FrameType::Color, FrameType::Ir, FrameType::Depth]
    } else {
      &[FrameType::Ir, FrameType::Depth]
    };

    let frame_listener = FRAME_LISTENER.get_or_try_init(|| MultiFrameListener::new(frame_types))?;

    device.set_ir_and_depth_frame_listener(frame_listener)?;
    device.set_color_frame_listener(frame_listener)?;
    let config = CONFIG.get_or_try_init(|| -> anyhow::Result<Config> {
      let mut config = Config::new()?;
      config.set_max_depth(MAX_DEPTH)?;

      Ok(config)
    })?;
    device.set_config(config)?;

    device.start_streams(render_type.is_color(), true)?;

    Ok(Self {
      registration: device.get_registration()?,
      render_type,
      _device: device,
    })
  }

  pub fn get_frame(&self) -> anyhow::Result<FrameMap<OwnedFrame>> {
    let mut frames = FRAME_LISTENER
      .get()
      .ok_or(anyhow!("Frame listener not initialized"))?
      .get_frames()?;

    if self.render_type.is_color() {
      let depth = frames.expect_depth()?;
      let color = frames.expect_color()?;

      let mut undistorted = Frame::depth();
      let mut color_depth = Frame::color_for_depth();
      if self.render_type == RenderType::Color {
        self.registration.map_depth_to_color(
          depth,
          color,
          &mut undistorted,
          &mut color_depth,
          true,
        )?;

        frames.set_depth(undistorted.to_owned());
        frames.set_color(color_depth.to_owned());
      } else {
        let mut full_color = Frame::depth_full_color();

        self.registration.map_depth_to_full_color(
          depth,
          color,
          &mut undistorted,
          &mut color_depth,
          true,
          &mut full_color,
        )?;

        frames.set_depth(full_color.to_owned());
      }
    } else {
      let depth = frames.expect_depth()?;

      let mut undistorted = Frame::depth();
      self
        .registration
        .undistort_depth(&depth.to_frame(), &mut undistorted)?;

      frames.set_depth(undistorted.to_owned());
    }

    Ok(frames)
  }
}
