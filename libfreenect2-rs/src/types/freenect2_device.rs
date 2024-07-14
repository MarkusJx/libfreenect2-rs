use anyhow::anyhow;
use cxx::UniquePtr;

use crate::ffi;
use crate::types::config::Config;
use crate::types::frame_listener::FrameListener;

pub struct Freenect2Device<'a>(UniquePtr<ffi::libfreenect2::Freenect2Device<'a>>);

impl<'a> Freenect2Device<'a> {
  pub(crate) fn new(inner: UniquePtr<ffi::libfreenect2::Freenect2Device<'a>>) -> Self {
    Self(inner)
  }

  pub fn get_serial_number(&mut self) -> anyhow::Result<String> {
    unsafe {
      self
        .0
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .get_serial_number()
        .map_err(Into::into)
    }
  }

  pub fn get_firmware_version(&mut self) -> anyhow::Result<String> {
    unsafe {
      self
        .0
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .get_firmware_version()
        .map_err(Into::into)
    }
  }

  pub fn start(&mut self) -> anyhow::Result<bool> {
    unsafe {
      self
        .0
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .start()
        .map_err(Into::into)
    }
  }

  pub fn start_streams(&mut self, color: bool, depth: bool) -> anyhow::Result<bool> {
    unsafe {
      self
        .0
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .start_streams(color, depth)
        .map_err(Into::into)
    }
  }

  pub fn stop(&mut self) -> anyhow::Result<bool> {
    unsafe {
      self
        .0
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .stop()
        .map_err(Into::into)
    }
  }

  pub fn close(&mut self) -> anyhow::Result<bool> {
    unsafe {
      self
        .0
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .close()
        .map_err(Into::into)
    }
  }

  pub fn set_color_frame_listener(
    &mut self,
    listener: &'a FrameListener<'a>,
  ) -> anyhow::Result<()> {
    unsafe {
      self
        .0
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .set_color_frame_listener(&listener.0)
        .map_err(Into::into)
    }
  }

  pub fn set_ir_and_depth_frame_listener(
    &mut self,
    listener: &'a FrameListener<'a>,
  ) -> anyhow::Result<()> {
    unsafe {
      self
        .0
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .set_ir_and_depth_frame_listener(&listener.0)
        .map_err(Into::into)
    }
  }

  pub fn set_config(&mut self, config: &'a Config) -> anyhow::Result<()> {
    unsafe {
      self
        .0
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .set_config(&config.0)
        .map_err(Into::into)
    }
  }
}
