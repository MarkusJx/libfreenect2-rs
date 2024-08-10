use libfreenect2_rs::config::Config;
use libfreenect2_rs::frame_listener::FrameListener;
use libfreenect2_rs::freenect2::Freenect2;
use libfreenect2_rs::freenect2_device::{LedMode, LedSettings};
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

fn main() -> anyhow::Result<()> {
  log4rs::init_config(
    log4rs::Config::builder()
      .appender(Appender::builder().build("stdout", Box::new(ConsoleAppender::builder().build())))
      .build(Root::builder().appender("stdout").build(LevelFilter::Trace))?,
  )?;

  let mut freenect2 = Freenect2::new()?;
  log::info!(
    "Number of connected devices: {}",
    freenect2.enumerate_devices()?
  );

  let mut config = Config::new()?;
  let frame_counts = Arc::new(Mutex::new(HashMap::new()));
  let listener = FrameListener::new(|ty, _frame| {
    let mut lock = frame_counts.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    lock.entry(ty).and_modify(|e| *e += 1).or_insert(1);

    Ok(())
  })?;

  let mut device = freenect2.open_default_device()?;
  log::info!("Serial number: {}", device.get_serial_number()?);

  config.set_max_depth(6.5)?;
  device.set_config(&config)?;
  device.set_color_frame_listener(&listener)?;
  device.set_ir_and_depth_frame_listener(&listener)?;

  device.start()?;
  device.set_led_settings(&LedSettings {
    id: 1,
    mode: LedMode::Blink,
    start_level: 0,
    stop_level: 1000,
    interval_ms: 1000,
  })?;

  log::info!("Sleeping for 5 seconds...");
  std::thread::sleep(std::time::Duration::from_secs(5));

  device.stop()?;

  let frame_counts = frame_counts.lock().unwrap();
  for (ty, count) in frame_counts.iter() {
    log::info!(
      "Received {} frames of type {:?}, {}FPS",
      count,
      ty,
      *count as f64 / 5.0
    );
  }

  Ok(())
}
