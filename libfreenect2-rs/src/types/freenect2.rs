use anyhow::anyhow;
use cxx::UniquePtr;

use crate::ffi;
use crate::types::freenect2_device::Freenect2Device;

pub struct Freenect2(UniquePtr<ffi::libfreenect2::Freenect2>);

impl Freenect2 {
  pub fn new() -> anyhow::Result<Self> {
    ffi::libfreenect2::create_freenect2()
      .map(Self)
      .map_err(Into::into)
  }

  pub fn enumerate_devices(&mut self) -> anyhow::Result<i32> {
    self
      .0
      .as_mut()
      .ok_or(anyhow!("Could not get freenect2 as mutable"))?
      .enumerate_devices()
      .map_err(Into::into)
  }

  pub fn get_device_serial_number(&mut self, idx: i32) -> anyhow::Result<String> {
    self
      .0
      .as_mut()
      .ok_or(anyhow!("Could not get freenect2 as mutable"))?
      .get_device_serial_number(idx)
      .map_err(Into::into)
  }

  pub fn get_default_device_serial_number(&mut self) -> anyhow::Result<String> {
    self
      .0
      .as_mut()
      .ok_or(anyhow!("Could not get freenect2 as mutable"))?
      .get_default_device_serial_number()
      .map_err(Into::into)
  }

  pub fn open_default_device(&mut self) -> anyhow::Result<Freenect2Device> {
    unsafe {
      self
        .0
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 as mutable"))?
        .open_default_device()
        .map(Freenect2Device::new)
        .map_err(Into::into)
    }
  }

  pub fn open_device_by_id(&mut self, idx: i32) -> anyhow::Result<Freenect2Device> {
    unsafe {
      self
        .0
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 as mutable"))?
        .open_device_by_id(idx)
        .map(Freenect2Device::new)
        .map_err(Into::into)
    }
  }

  pub fn open_device_by_serial(&mut self, serial: &str) -> anyhow::Result<Freenect2Device> {
    unsafe {
      self
        .0
        .as_mut()
        .ok_or(anyhow!("Could not get freenect2 as mutable"))?
        .open_device_by_serial(serial)
        .map(Freenect2Device::new)
        .map_err(Into::into)
    }
  }
}

unsafe impl Send for Freenect2 {}
unsafe impl Sync for Freenect2 {}
