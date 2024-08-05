use crate::ffi;
use crate::types::freenect2_device::Freenect2Device;
use anyhow::{anyhow, Context, Error};
use cxx::UniquePtr;
use std::pin::Pin;
use std::sync::{Arc, LazyLock, Mutex, MutexGuard, Weak};

/// Global static instance of `Freenect2`.
/// Stored in a [`LazyLock`] to ensure it is only created once.
/// Stored in a [`Mutex`] to allow for interior mutability.
/// Stored in a [`Weak`] to allow for dropping when no longer needed.
static FREENECT2: LazyLock<Mutex<Weak<Mutex<Freenect2Impl>>>> = LazyLock::new(Mutex::default);

struct Freenect2Impl(UniquePtr<ffi::libfreenect2::Freenect2>);

impl Freenect2Impl {
  fn new() -> anyhow::Result<Self> {
    unsafe { ffi::libfreenect2::create_freenect2() }
      .context("Failed to create freenect2")
      .map(Self)
  }

  fn get_mut(&mut self) -> anyhow::Result<Pin<&mut ffi::libfreenect2::Freenect2>> {
    self
      .0
      .as_mut()
      .ok_or(anyhow!("Failed to get freenect2 as mutable"))
  }
}

unsafe impl Send for Freenect2Impl {}
unsafe impl Sync for Freenect2Impl {}

/// Wrapper around libfreenect2's `Freenect2` class.
/// Used to manage the connected devices.
/// The `Freenect2` instance is used to open devices.
///
/// # Example
/// ```no_run
/// use libfreenect2_rs::freenect2::Freenect2;
///
/// let mut freenect2 = Freenect2::new().unwrap();
/// let num_devices = freenect2.enumerate_devices().unwrap();
/// println!("Found {} devices", num_devices);
///
/// let serial = freenect2.get_default_device_serial_number().unwrap();
/// println!("Default device serial number: {}", serial);
///
/// let mut device = freenect2.open_default_device().unwrap();
/// let serial = device.get_serial_number().unwrap();
/// println!("Opened device with serial number: {}", serial);
/// ```
pub struct Freenect2(Arc<Mutex<Freenect2Impl>>);

impl Freenect2 {
  /// Create a new `Freenect2` instance.
  ///
  /// Uses a global static instance stored in a [`Weak`]
  /// to ensure only one instance is created. If another
  /// instance is already alive, it will be returned.
  /// Otherwise, a new instance will be created.
  /// The instance will be dropped when the last reference
  /// is dropped.
  pub fn new() -> anyhow::Result<Self> {
    let mut static_instance = FREENECT2.lock().unwrap();

    if let Some(instance) = static_instance.upgrade() {
      return Ok(Self(instance));
    }

    let instance = Arc::new(Mutex::new(Freenect2Impl::new()?));
    *static_instance = Arc::downgrade(&instance);

    Ok(Self(instance))
  }

  /// Enumerate the connected devices.
  /// Returns the number of devices found.
  ///
  /// # Errors
  /// Returns an error if the underlying C++ function fails.
  pub fn enumerate_devices(&mut self) -> anyhow::Result<i32> {
    let mut this = self.get_mut()?;
    this.get_mut()?.enumerate_devices().map_err(Into::into)
  }

  /// Get the serial number of the device at the specified index.
  /// Returns the serial number as a string.
  ///
  /// # Arguments
  /// * `idx` - The index of the device to get the serial number of.
  ///
  /// # Errors
  /// Returns an error if the device at the specified index is not found.
  pub fn get_device_serial_number(&mut self, idx: i32) -> anyhow::Result<String> {
    let mut this = self.get_mut()?;
    this
      .get_mut()?
      .get_device_serial_number(idx)
      .map_err(Into::into)
  }

  /// Get the serial number of the default device.
  /// Returns the serial number as a string.
  ///
  /// # Errors
  /// Returns an error if no default device is found.
  pub fn get_default_device_serial_number(&mut self) -> anyhow::Result<String> {
    let mut this = self.get_mut()?;
    this
      .get_mut()?
      .get_default_device_serial_number()
      .map_err(Into::into)
  }

  /// Open the default device.
  /// Returns a new [`Freenect2Device`] instance.
  ///
  /// # Errors
  /// Returns an error if no default device is found.
  pub fn open_default_device(&mut self) -> anyhow::Result<Freenect2Device> {
    let mut this = self.get_mut()?;
    unsafe {
      this
        .get_mut()?
        .open_default_device()
        .map(Freenect2Device::new)
        .map_err(Into::into)
    }
  }

  /// Open the device at the specified index.
  /// Returns a new [`Freenect2Device`] instance.
  ///
  /// # Arguments
  /// * `idx` - The index of the device to open.
  ///
  /// # Errors
  /// Returns an error if the device at the specified index is not found.
  pub fn open_device_by_id(&mut self, idx: i32) -> anyhow::Result<Freenect2Device> {
    let mut this = self.get_mut()?;
    unsafe {
      this
        .get_mut()?
        .open_device_by_id(idx)
        .map(Freenect2Device::new)
        .map_err(Into::into)
    }
  }

  /// Open the device with the specified serial number.
  ///
  /// # Arguments
  /// * `serial` - The serial number of the device to open.
  ///
  /// # Errors
  /// Returns an error if the device with the specified serial number is not found.
  pub fn open_device_by_serial(&mut self, serial: &str) -> anyhow::Result<Freenect2Device> {
    let mut this = self.get_mut()?;
    unsafe {
      this
        .get_mut()?
        .open_device_by_serial(serial)
        .map(Freenect2Device::new)
        .map_err(Into::into)
    }
  }

  #[cfg(test)]
  pub(crate) fn has_instance() -> bool {
    FREENECT2.lock().unwrap().upgrade().is_some()
  }

  fn get_mut(&mut self) -> Result<MutexGuard<'_, Freenect2Impl>, Error> {
    self
      .0
      .lock()
      .map_err(|e| anyhow!(e.to_string()))
      .context("Failed to get freenect2 as mutable")
  }
}

unsafe impl Send for Freenect2 {}
unsafe impl Sync for Freenect2 {}
