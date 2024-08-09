use crate::constants::{FINAL_Z_SCALE, SCALE};
use crate::renderer::PointCloudRenderer;
use libfreenect2_rs::frame::Freenect2Frame;
use libfreenect2_rs::frame_data::{FloatData, RGBXData};
use na::{min, Point3};
use rayon::prelude::*;
use std::sync::Mutex;

pub struct MultiThreadImageProcessor {
  data: Vec<Mutex<Vec<Point3<f32>>>>,
  num_threads: usize,
}

impl MultiThreadImageProcessor {
  pub fn new(num_threads: usize) -> Self {
    log::debug!(
      "Creating multi-threaded image processor with {} threads",
      num_threads
    );

    Self {
      data: (0..num_threads).map(|_| Mutex::new(Vec::new())).collect(),
      num_threads,
    }
  }

  pub fn process(
    &self,
    depth_data: &FloatData,
    color_data: &RGBXData,
    color: &dyn Freenect2Frame,
    y_scale: f32,
    depth_offset: usize,
    renderer: &mut PointCloudRenderer,
  ) {
    (0..self.num_threads)
      .into_par_iter()
      .for_each(|i| self.calculate_points(depth_data, color_data, color, y_scale, depth_offset, i));

    for i in 0..self.num_threads {
      let mut data = self.data[i].lock().unwrap();
      renderer.append(&mut data);
    }
  }

  fn calculate_points(
    &self,
    depth_data: &FloatData,
    color_data: &RGBXData,
    color: &dyn Freenect2Frame,
    y_scale: f32,
    depth_offset: usize,
    index: usize,
  ) {
    let mut data = self.data[index].lock().unwrap();
    let start_y = (color.height() / self.num_threads) * index;
    let end_y = min(
      (color.height() / self.num_threads) * (index + 1),
      color.height(),
    );

    for y in start_y..end_y {
      for x in 0..color.width() {
        let Some(z) = depth_data.get_valid_pixel(x, y + depth_offset) else {
          continue;
        };

        let x_f32 = x as f32 / (color.width() as f32) - 0.5;
        let y_f32 = 0.5 - y as f32 / (color.height() as f32);

        data.push(Point3::new(
          x_f32 * SCALE,
          y_f32 * SCALE * y_scale,
          z * FINAL_Z_SCALE,
        ));
        data.push(Point3::new(
          color_data.red_at(x, y) as f32 / 255.0,
          color_data.green_at(x, y) as f32 / 255.0,
          color_data.blue_at(x, y) as f32 / 255.0,
        ));
      }
    }
  }
}
