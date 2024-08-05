use anyhow::anyhow;
use cxx::UniquePtr;

use crate::ffi;
use crate::frame_listener::AsFrameListener;
use crate::types::config::Config;
use crate::types::registration::Registration;

/// Wrapper around libfreenect2's `Freenect2Device` class.
/// Used to manage a single connected device.
/// The `Freenect2Device` instance is used to start and stop streams,
/// set listeners, and get device information.
pub struct Freenect2Device<'a> {
  device: UniquePtr<ffi::libfreenect2::Freenect2Device<'a>>,
  started: bool,
  closed: bool,
}

impl<'a> Freenect2Device<'a> {
  pub(crate) fn new(device: UniquePtr<ffi::libfreenect2::Freenect2Device<'a>>) -> Self {
    Self {
      device,
      started: false,
      closed: false,
    }
  }

  /// Get the serial number of the device.
  /// This is a unique identifier for the device.
  ///
  /// # Errors
  /// Returns an error if the serial number could not be retrieved.
  pub fn get_serial_number(&mut self) -> anyhow::Result<String> {
    anyhow::ensure!(
      !self.closed,
      "Device must not be closed when getting the serial number"
    );

    unsafe {
      self
        .device
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .get_serial_number()
        .map_err(Into::into)
    }
  }

  /// Get the firmware version of the device.
  ///
  /// # Errors
  /// Returns an error if the firmware version could not be retrieved.
  pub fn get_firmware_version(&mut self) -> anyhow::Result<String> {
    anyhow::ensure!(
      !self.closed,
      "Device must not be closed when getting the firmware version"
    );

    unsafe {
      self
        .device
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .get_firmware_version()
        .map_err(Into::into)
    }
  }

  /// Start the device with depth and color streams enabled.
  /// For more control over the streams, use [`Self::start_streams`] instead.
  ///
  /// # Errors
  /// Returns an error if the device could not be started.
  pub fn start(&mut self) -> anyhow::Result<()> {
    anyhow::ensure!(
      !self.closed,
      "Device must not be closed when starting streams"
    );
    anyhow::ensure!(
      !self.started,
      "Device must be stopped before starting streams"
    );

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

  /// Start the device with the specified streams enabled.
  /// The `color` and `depth` arguments specify whether the color and depth streams should be enabled.
  ///
  /// # Arguments
  /// * `color` - Whether to enable the color stream.
  /// * `depth` - Whether to enable the depth stream.
  ///
  /// # Errors
  /// Returns an error if the device could not be started or both `color` and `depth` are `false`.
  pub fn start_streams(&mut self, color: bool, depth: bool) -> anyhow::Result<()> {
    anyhow::ensure!(color || depth, "At least one stream must be enabled");
    anyhow::ensure!(
      !self.closed,
      "Device must not be closed when starting streams"
    );
    anyhow::ensure!(
      !self.started,
      "Device must be stopped before starting streams"
    );

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

  /// Stop the device and streams.
  /// The device must be started before stopping streams.
  ///
  /// # Errors
  /// Returns an error if the device is not started or the streams could not be stopped.
  pub fn stop(&mut self) -> anyhow::Result<()> {
    anyhow::ensure!(
      !self.closed,
      "Device must not be closed when stopping streams"
    );
    anyhow::ensure!(
      self.started,
      "Device must be started before stopping streams"
    );

    unsafe {
      self
        .device
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .stop()
        .map_err(Into::into)
        .and_then(|stopped| {
          anyhow::ensure!(stopped, "Failed to stop streams");
          self.started = !stopped;

          Ok(())
        })
    }
  }

  /// Close the device.
  /// After closing the device, it cannot be used again.
  ///
  /// # Errors
  /// Returns an error if the device could not be closed or is already closed.
  pub fn close(&mut self) -> anyhow::Result<()> {
    anyhow::ensure!(
      !self.closed,
      "Device must not be closed when closing the device"
    );

    unsafe {
      self
        .device
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .close()
        .map_err(Into::into)
        .and_then(|closed| {
          anyhow::ensure!(closed, "Failed to close device");
          self.closed = closed;

          Ok(())
        })
    }
  }

  /// Set the color frame listener.
  /// The listener will be called when a new color frame is available.
  ///
  /// # Arguments
  /// * `listener` - The listener to set.
  ///
  /// # Errors
  /// Returns an error if the listener could not be set.
  pub fn set_color_frame_listener<L: AsFrameListener<'a>>(
    &mut self,
    listener: &'a L,
  ) -> anyhow::Result<()> {
    anyhow::ensure!(
      !self.closed,
      "Device must not be closed when setting listener"
    );

    unsafe {
      self
        .device
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .set_color_frame_listener(&listener.as_frame_listener().0)
        .map_err(Into::into)
    }
  }

  /// Set the IR and depth frame listener.
  /// The listener will be called when a new IR and depth frame is available.
  ///
  /// # Arguments
  /// * `listener` - The listener to set.
  ///
  /// # Errors
  /// Returns an error if the listener could not be set.
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

  /// Set the configuration for the device.
  /// The configuration specifies the resolution and format of the streams.
  ///
  /// # Arguments
  /// * `config` - The configuration to set.
  ///   Use [`Config::new`] to create a new configuration.
  ///
  /// # Errors
  /// Returns an error if the configuration could not
  /// be set or the device is already started.
  pub fn set_config(&mut self, config: &'a Config) -> anyhow::Result<()> {
    anyhow::ensure!(
      !self.closed,
      "Device must not be closed when setting config"
    );
    anyhow::ensure!(
      !self.started,
      "Device must be stopped before setting config"
    );

    unsafe {
      self
        .device
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 device as mutable"))?
        .set_config(&config.0)
        .map_err(Into::into)
    }
  }

  /// Get the registration for the device.
  /// The registration is used to map depth frames to color frames.
  /// The device must be started before getting the registration.
  ///
  /// # Errors
  /// Returns an error if the registration could not be retrieved or the device is not started.
  pub fn get_registration(&mut self) -> anyhow::Result<Registration> {
    anyhow::ensure!(
      !self.closed,
      "Device must not be closed when getting registration"
    );
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
