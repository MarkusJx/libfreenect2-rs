use crate::constants::{FINAL_Z_SCALE, SCALE, Z_HUE_SCALE};
use crate::freenect_state::FreenectState;
use crate::renderer::PointCloudRenderer;
use crate::RenderType;
use colors_transform::{Color, Hsl};
use kiss3d::camera::Camera;
use kiss3d::planar_camera::PlanarCamera;
use kiss3d::post_processing::PostProcessingEffect;
use kiss3d::renderer::Renderer;
use kiss3d::text::Font;
use kiss3d::window::{State, Window};
use libfreenect2_rs::frame::Freenect2Frame;
use libfreenect2_rs::frame_data::FrameDataValue;
use na::{Point2, Point3};
use std::time::Instant;

pub struct AppState {
  point_cloud_renderer: PointCloudRenderer,
  last_render_start: Instant,
  freenect_state: FreenectState<'static>,
  i: usize,
  render_type: RenderType,
  last_fps: Option<String>,
}

impl AppState {
  pub fn new(
    freenect_state: FreenectState<'static>,
    render_type: RenderType,
  ) -> anyhow::Result<Self> {
    Ok(Self {
      point_cloud_renderer: PointCloudRenderer::new(4.0),
      last_render_start: Instant::now(),
      freenect_state,
      i: 0,
      render_type,
      last_fps: None,
    })
  }

  fn draw_fps(&mut self, window: &mut Window) {
    window.draw_text(
      self.last_fps.as_ref().unwrap_or(&"".to_string()),
      &Point2::new(10.0, 20.0),
      60.0,
      &Font::default(),
      &Point3::new(1.0, 1.0, 1.0),
    );
  }
}

impl State for AppState {
  fn step(&mut self, window: &mut Window) {
    let elapsed = self.last_render_start.elapsed();
    let render_start = Instant::now();

    let frames = self.freenect_state.get_frame().unwrap();
    let frame_fetch = render_start.elapsed();

    self.i += 1;
    /*if self.render_type == RenderType::FullColor && self.i % 2 != 0 {
      drop(frames);
      self.draw_fps(window);
      return;
    }*/

    self.last_render_start = render_start;
    self.point_cloud_renderer.clear();

    let depth = frames.expect_depth().unwrap();
    // Correct the aspect ratio of the frame since we are drawing a square.
    let y_scale = depth.height() as f32 / depth.width() as f32;

    let start = Instant::now();
    if self.render_type.is_color() {
      let color = frames.expect_color().unwrap();

      // The first line of the depth frame is empty if the depth frame is
      // in the same frame as the color frame (1920x1080).
      let depth_offset = self.render_type.depth_offset();
      let depth_data = depth.data().expect_float();
      let color_data = color.data().expect_rgbx();

      for y in 0..color.height() {
        for x in 0..color.width() {
          let Some(z) = depth_data.get_valid_pixel(x, y + depth_offset) else {
            continue;
          };

          let x_f32 = x as f32 / (color.width() as f32) - 0.5;
          let y_f32 = 0.5 - y as f32 / (color.height() as f32);

          self.point_cloud_renderer.push(
            Point3::new(x_f32 * SCALE, y_f32 * SCALE * y_scale, z * FINAL_Z_SCALE),
            Point3::new(
              color_data.red_at(x, y) as f32 / 255.0,
              color_data.green_at(x, y) as f32 / 255.0,
              color_data.blue_at(x, y) as f32 / 255.0,
            ),
          );
        }
      }
    } else {
      let width = depth.width() as f32;
      let height = depth.height() as f32;
      for (y, row) in depth.data().expect_float().iter().enumerate() {
        for (x, z) in row.enumerate() {
          if z <= 0.0 {
            continue;
          }

          let x_f32 = x as f32 / width - 0.5;
          let y_f32 = 0.5 - y as f32 / height;

          let color = Hsl::from(z * Z_HUE_SCALE, 100.0, 50.0);

          self.point_cloud_renderer.push(
            Point3::new(x_f32 * SCALE, y_f32 * SCALE * y_scale, z * FINAL_Z_SCALE),
            Point3::new(
              color.get_red() / 255.0,
              color.get_green() / 255.0,
              color.get_blue() / 255.0,
            ),
          );
        }
      }
    }

    let processing = start.elapsed();
    if self.i % 15 == 0 {
      self.last_fps = Some(format!(
        "{} FPS, processing: {}ms, full render: {}ms, frame fetch: {}ms",
        1_000_000 / elapsed.as_micros(),
        processing.as_millis(),
        elapsed.as_millis(),
        frame_fetch.as_millis()
      ));
    }

    drop(frames);
    self.draw_fps(window);
  }

  // Return the custom renderer that will be called at each
  // render loop.
  fn cameras_and_effect_and_renderer(
    &mut self,
  ) -> (
    Option<&mut dyn Camera>,
    Option<&mut dyn PlanarCamera>,
    Option<&mut dyn Renderer>,
    Option<&mut dyn PostProcessingEffect>,
  ) {
    (None, None, Some(&mut self.point_cloud_renderer), None)
  }
}
