use anyhow::anyhow;
use cxx::UniquePtr;

use crate::ffi;

pub struct Config(pub(crate) UniquePtr<ffi::libfreenect2::Config>);

impl Config {
  pub fn new() -> anyhow::Result<Self> {
    ffi::libfreenect2::create_config()
      .map(Self)
      .map_err(Into::into)
  }

  pub fn get_min_depth(&self) -> f32 {
    self.0.get_min_depth()
  }

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

  pub fn get_max_depth(&self) -> f32 {
    self.0.get_max_depth()
  }

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

  pub fn get_enable_bilateral_filter(&self) -> bool {
    self.0.get_enable_bilateral_filter()
  }

  pub fn set_enable_bilateral_filter(&mut self, enable: bool) -> anyhow::Result<&mut Self> {
    self
      .0
      .as_mut()
      .ok_or(anyhow!("Failed to get config as mutable"))?
      .set_enable_bilateral_filter(enable);
    Ok(self)
  }

  pub fn get_enable_edge_aware_filter(&self) -> bool {
    self.0.get_enable_edge_aware_filter()
  }

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
