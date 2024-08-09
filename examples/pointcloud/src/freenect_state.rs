use crate::constants::MAX_DEPTH;
use crate::RenderType;
use anyhow::anyhow;
use libfreenect2_rs::config::Config;
use libfreenect2_rs::frame::{Frame, FrameReference};
use libfreenect2_rs::frame_listener::{FrameMap, MultiFrameListener};
use libfreenect2_rs::frame_type::FrameType;
use libfreenect2_rs::freenect2::Freenect2;
use libfreenect2_rs::freenect2_device::Freenect2Device;
use libfreenect2_rs::registration::Registration;
use once_cell::sync::OnceCell;

pub type FrameListenerType = Frame<'static>;

static FRAME_LISTENER: OnceCell<MultiFrameListener<'static, FrameListenerType>> = OnceCell::new();
static mut FREENECT: Option<Freenect2> = None;
static CONFIG: OnceCell<Config> = OnceCell::new();

pub struct FreenectState<'a> {
  _device: Freenect2Device<'a>,
  registration: Registration,
  render_type: RenderType,
  undistorted: Frame<'static>,
  color_depth: Frame<'static>,
  full_color: Frame<'static>,
}

impl FreenectState<'_> {
  pub fn new(render_type: RenderType) -> anyhow::Result<Self> {
    let mut device = unsafe {
      FREENECT = Some(Freenect2::new()?);
      FREENECT.as_mut().unwrap().open_default_device()?
    };

    let frame_types: &[FrameType] = if render_type.is_color() {
      &[FrameType::Color, FrameType::Depth]
    } else {
      &[FrameType::Depth]
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
      undistorted: Frame::depth(),
      color_depth: Frame::color_for_depth(),
      full_color: Frame::depth_full_color(),
    })
  }

  pub fn get_frame(&mut self) -> anyhow::Result<FrameMap<FrameReference<'_, 'static>>> {
    let mut frames = FRAME_LISTENER
      .get()
      .ok_or(anyhow!("Frame listener not initialized"))?
      .get_frames()?;

    let mut res_frames = FrameMap::default();
    if self.render_type.is_color() {
      let depth = frames.expect_depth()?;
      let color = frames.expect_color()?;

      if self.render_type == RenderType::Color {
        self.registration.map_depth_to_color(
          depth,
          color,
          &mut self.undistorted,
          &mut self.color_depth,
          true,
        )?;

        res_frames.set_depth(FrameReference::Borrowed(&self.undistorted));
        res_frames.set_color(FrameReference::Borrowed(&self.color_depth));
      } else {
        self.registration.map_depth_to_full_color(
          depth,
          color,
          &mut self.undistorted,
          &mut self.color_depth,
          true,
          &mut self.full_color,
        )?;

        res_frames.set_depth(FrameReference::Borrowed(&self.full_color));
        res_frames.set_color(frames.expect_take_color()?);
      }
    } else {
      let depth = frames.expect_depth()?;

      self
        .registration
        .undistort_depth(depth, &mut self.undistorted)?;

      res_frames.set_depth(FrameReference::Borrowed(&self.undistorted));
    }

    Ok(res_frames)
  }
}
