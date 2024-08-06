use crate::ffi::libfreenect2::create_logger;
use std::sync::Once;

static LOGGER_INIT: Once = Once::new();

pub(crate) fn init_logger() {
  LOGGER_INIT.call_once(|| {
    create_logger(|level, message| match level {
      crate::ffi::libfreenect2::LogLevel::Error => log::error!("{}", message),
      crate::ffi::libfreenect2::LogLevel::Warning => log::warn!("{}", message),
      crate::ffi::libfreenect2::LogLevel::Info => log::info!("{}", message),
      crate::ffi::libfreenect2::LogLevel::Debug => log::debug!("{}", message),
      _ => log::error!("Unknown log level: {}", message),
    })
    .unwrap();
  });
}
