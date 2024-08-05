use anyhow::anyhow;
use cxx::UniquePtr;

use crate::ffi;
use crate::frame_listener::AsFrameListener;
use crate::types::config::Config;
use crate::types::registration::Registration;

pub struct Freenect2Device<'a> {
  device: UniquePtr<ffi::libfreenect2::Freenect2Device<'a>>,
  started: bool,
}

impl<'a> Freenect2Device<'a> {
  pub(crate) fn new(device: UniquePtr<ffi::libfreenect2::Freenect2Device<'a>>) -> Self {
    Self {
      device,
      started: false,
    }
  }

  pub fn get_serial_number(&mut self) -> anyhow::Result<String> {
    unsafe {
      self
        .device
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .get_serial_number()
        .map_err(Into::into)
    }
  }

  pub fn get_firmware_version(&mut self) -> anyhow::Result<String> {
    unsafe {
      self
        .device
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .get_firmware_version()
        .map_err(Into::into)
    }
  }

  pub fn start(&mut self) -> anyhow::Result<()> {
    unsafe {
      self
        .device
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .start()
        .map_err(Into::into)
        .and_then(|started| {
          anyhow::ensure!(started, "Failed to start streams");
          self.started = started;
          Ok(())
        })
    }
  }

  pub fn start_streams(&mut self, color: bool, depth: bool) -> anyhow::Result<()> {
    unsafe {
      self
        .device
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .start_streams(color, depth)
        .map_err(Into::into)
        .and_then(|started| {
          anyhow::ensure!(started, "Failed to start streams");
          self.started = started;
          Ok(())
        })
    }
  }

  pub fn stop(&mut self) -> anyhow::Result<bool> {
    unsafe {
      self
        .device
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .stop()
        .map_err(Into::into)
    }
  }

  pub fn close(&mut self) -> anyhow::Result<bool> {
    unsafe {
      self
        .device
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .close()
        .map_err(Into::into)
    }
  }

  pub fn set_color_frame_listener<L: AsFrameListener<'a>>(
    &mut self,
    listener: &'a L,
  ) -> anyhow::Result<()> {
    unsafe {
      self
        .device
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .set_color_frame_listener(&listener.as_frame_listener().0)
        .map_err(Into::into)
    }
  }

  pub fn set_ir_and_depth_frame_listener<L: AsFrameListener<'a>>(
    &mut self,
    listener: &'a L,
  ) -> anyhow::Result<()> {
    unsafe {
      self
        .device
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .set_ir_and_depth_frame_listener(&listener.as_frame_listener().0)
        .map_err(Into::into)
    }
  }

  pub fn set_config(&mut self, config: &'a Config) -> anyhow::Result<()> {
    unsafe {
      self
        .device
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .set_config(&config.0)
        .map_err(Into::into)
    }
  }

  pub fn get_registration(&mut self) -> anyhow::Result<Registration> {
    anyhow::ensure!(
      self.started,
      "Device must be started before getting registration"
    );

    unsafe {
      self
        .device
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .get_registration()
        .map(Registration::new)
        .map_err(Into::into)
    }
  }
}
