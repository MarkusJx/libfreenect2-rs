use crate::types::frame::{Frame, Freenect2Frame};
#[cfg(test)]
use crate::types::frame_data::FrameData;
#[cfg(test)]
use crate::types::frame_listener::FrameListener;
use crate::types::frame_type::FrameType;
#[cfg(test)]
use crate::types::freenect2::Freenect2;
#[cfg(test)]
use std::sync::atomic::{AtomicUsize, Ordering};
#[cfg(test)]
use std::time::{Duration, Instant};

pub mod ffi;
pub mod types;

#[test]
fn test() {
  let mut freenect2 = Freenect2::new().unwrap();
  let devs = freenect2.enumerate_devices().unwrap();

  println!("Found {} devices", devs);

  let serial = freenect2.get_default_device_serial_number().unwrap();
  println!("Default device serial number: {}", serial);

  let color_frames = AtomicUsize::default();
  let ir_frames = AtomicUsize::default();
  let depth_frames = AtomicUsize::default();

  let listener = FrameListener::new(|i, frame| {
    match i {
      FrameType::Color => {
        color_frames.fetch_add(1, Ordering::SeqCst);
      }
      FrameType::Ir => {
        ir_frames.fetch_add(1, Ordering::SeqCst);
      }
      FrameType::Depth => {
        depth_frames.fetch_add(1, Ordering::SeqCst);
      }
    };

    println!(
      "Frame: {i:?}, {}x{}, {} bytes per pixel, {:?}",
      frame.width(),
      frame.height(),
      frame.bytes_per_pixel(),
      frame.timestamp_as_duration()
    );

    let now = Instant::now();
    /*let data = frame.raw_data();
    match data {
      FrameData::Raw(_) => {
        println!("Raw data");
      }
      FrameData::Float(_) => {
        println!("Float data");
      }
      FrameData::RGBX(rgbx) => {
        println!("RGBX data: {}", rgbx.len());
      }
      FrameData::Gray(_) => {
        println!("Gray data");
      }
    }*/

    println!("Data processing took: {:?}", now.elapsed());
  })
  .unwrap();

  let mut device = freenect2.open_default_device().unwrap();

  device.set_color_frame_listener(&listener).unwrap();
  device.set_ir_and_depth_frame_listener(&listener).unwrap();

  device.start_streams(true, true).unwrap();
  std::thread::sleep(Duration::from_secs(5));

  drop(device);

  let color_frames = color_frames.load(Ordering::SeqCst);
  let ir_frames = ir_frames.load(Ordering::SeqCst);
  let depth_frames = depth_frames.load(Ordering::SeqCst);
  println!(
    "Color frames: {color_frames}, {} FPS",
    color_frames as f32 / 5f32
  );
  println!("IR frames: {ir_frames}, {} FPS", ir_frames as f32 / 5f32);
  println!(
    "Depth frames: {depth_frames}, {} FPS",
    depth_frames as f32 / 5f32
  );
}
