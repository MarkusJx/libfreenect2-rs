use anyhow::anyhow;
use cxx::UniquePtr;

use crate::ffi;

/// Configuration for the Kinect.
/// The config can be passed to [`crate::freenect2_device::Freenect2Device::set_config`]
/// to configure the device.
///
/// # Example
/// ```
/// use libfreenect2_rs::config::Config;
///
/// let mut config = Config::new().unwrap();
/// config.set_min_depth(0.5).unwrap();
/// config.set_max_depth(5.0).unwrap();
/// config.set_enable_bilateral_filter(true).unwrap();
/// config.set_enable_edge_aware_filter(true).unwrap();
///
/// assert_eq!(config.get_min_depth(), 0.5);
/// assert_eq!(config.get_max_depth(), 5.0);
/// assert_eq!(config.get_enable_bilateral_filter(), true);
/// assert_eq!(config.get_enable_edge_aware_filter(), true);
/// ```
pub struct Config(pub(crate) UniquePtr<ffi::libfreenect2::Config>);

impl Config {
  /// Create a new configuration.
  /// The configuration is initialized with the default values.
  /// The default values are:
  /// - Min depth: 0.5
  /// - Max depth: 4.5
  /// - Enable bilateral filter: true
  /// - Enable edge aware filter: true
  pub fn new() -> anyhow::Result<Self> {
    ffi::libfreenect2::create_config()
      .map(Self)
      .map_err(Into::into)
  }

  /// Get the minimum depth.
  pub fn get_min_depth(&self) -> f32 {
    self.0.get_min_depth()
  }

  /// Set the minimum depth.
  pub fn set_min_depth(&mut self, min_depth: f32) -> anyhow::Result<&mut Self> {
    anyhow::ensure!(min_depth > 0.0, "Min depth must be greater than 0");
    anyhow::ensure!(
      min_depth < self.get_max_depth(),
      "Min depth must be less than max depth"
    );

    self
      .0
      .as_mut()
      .ok_or(anyhow!("Failed to get config as mutable"))?
      .set_min_depth(min_depth);
    Ok(self)
  }

  /// Get the maximum depth.
  pub fn get_max_depth(&self) -> f32 {
    self.0.get_max_depth()
  }

  /// Set the maximum depth.
  pub fn set_max_depth(&mut self, max_depth: f32) -> anyhow::Result<&mut Self> {
    anyhow::ensure!(
      max_depth > self.get_min_depth(),
      "Max depth must be greater than min depth"
    );

    self
      .0
      .as_mut()
      .ok_or(anyhow!("Failed to get config as mutable"))?
      .set_max_depth(max_depth);
    Ok(self)
  }

  /// Get whether the bilateral filter is enabled.
  pub fn get_enable_bilateral_filter(&self) -> bool {
    self.0.get_enable_bilateral_filter()
  }

  /// Set whether the bilateral filter is enabled.
  /// This filter will remove some noise from the depth data.
  pub fn set_enable_bilateral_filter(&mut self, enable: bool) -> anyhow::Result<&mut Self> {
    self
      .0
      .as_mut()
      .ok_or(anyhow!("Failed to get config as mutable"))?
      .set_enable_bilateral_filter(enable);
    Ok(self)
  }

  /// Get whether the edge aware filter is enabled.
  pub fn get_enable_edge_aware_filter(&self) -> bool {
    self.0.get_enable_edge_aware_filter()
  }

  /// Set whether the edge aware filter is enabled.
  /// This filter will remove pixels on the edges because ToF cameras produce noisy edges.
  pub fn set_enable_edge_aware_filter(&mut self, enable: bool) -> anyhow::Result<&mut Self> {
    self
      .0
      .as_mut()
      .ok_or(anyhow!("Failed to get config as mutable"))?
      .set_enable_edge_aware_filter(enable);
    Ok(self)
  }
}

unsafe impl Send for Config {}
unsafe impl Sync for Config {}
