extern crate kiss3d;
extern crate nalgebra as na;

use colors_transform::{Color, Hsl};
use kiss3d::camera::Camera;
use kiss3d::context::Context;
use kiss3d::planar_camera::PlanarCamera;
use kiss3d::post_processing::PostProcessingEffect;
use kiss3d::renderer::Renderer;
use kiss3d::resource::{
  AllocationType, BufferType, Effect, GPUVec, ShaderAttribute, ShaderUniform,
};
use kiss3d::text::Font;
use kiss3d::window::{State, Window};
use libfreenect2_rs::types::config::Config;
use libfreenect2_rs::types::frame::{Freenect2Frame, OwnedFrame};
use libfreenect2_rs::types::frame_listener::FrameListener;
use libfreenect2_rs::types::frame_type::FrameType;
use libfreenect2_rs::types::freenect2::Freenect2;
use libfreenect2_rs::types::freenect2_device::Freenect2Device;
use na::{Matrix4, Point2, Point3};
use once_cell::sync::OnceCell;
use std::sync::mpsc::Receiver;
use std::time::Instant;

// Custom renderers are used to allow rendering objects that are not necessarily
// represented as meshes. In this example, we will render a large, growing, point cloud
// with a color associated to each point.

// Writing a custom renderer requires the main loop to be
// handled by the `State` trait instead of a `while window.render()`
// like other examples.

static FRAME_LISTENER: OnceCell<FrameListener<'static>> = OnceCell::new();
static mut FREENECT: Option<Freenect2> = None;
static CONFIG: OnceCell<Config> = OnceCell::new();
const MAX_DEPTH: f32 = 6.5;
const SCALE: f32 = 10.0;
const Z_SCALE: f32 = 3.0;

const FINAL_Z_SCALE: f32 = Z_SCALE / (MAX_DEPTH * 1000.0) * SCALE;
const Z_HUE_SCALE: f32 = 255.0 / (MAX_DEPTH * 1000.0);

struct FreenectState<'a> {
  _device: Freenect2Device<'a>,
  rx: Receiver<OwnedFrame>,
}

impl FreenectState<'_> {
  fn new() -> anyhow::Result<Self> {
    let mut device = unsafe {
      FREENECT = Some(Freenect2::new()?);
      FREENECT.as_mut().unwrap().open_default_device()?
    };

    let (tx, rx) = std::sync::mpsc::channel();
    let frame_listener = FRAME_LISTENER.get_or_try_init(|| {
      FrameListener::new(move |ty, frame| {
        if ty == FrameType::Depth {
          tx.send(frame.to_owned()).unwrap();
        }
      })
    })?;

    device.set_ir_and_depth_frame_listener(frame_listener)?;
    device.start_streams(false, true)?;

    let config = CONFIG.get_or_try_init(|| -> anyhow::Result<Config> {
      let mut config = Config::new()?;
      config.set_max_depth(MAX_DEPTH)?;

      Ok(config)
    })?;
    device.set_config(config)?;

    Ok(Self {
      _device: device,
      rx,
    })
  }

  fn get_frame(&self) -> anyhow::Result<OwnedFrame> {
    self.rx.recv().map_err(Into::into)
  }
}

struct AppState {
  point_cloud_renderer: PointCloudRenderer,
  last_render_start: Instant,
  freenect_state: FreenectState<'static>,
}

impl AppState {
  fn new(freenect_state: FreenectState<'static>) -> anyhow::Result<Self> {
    Ok(Self {
      point_cloud_renderer: PointCloudRenderer::new(4.0),
      last_render_start: Instant::now(),
      freenect_state,
    })
  }
}

impl State for AppState {
  fn step(&mut self, window: &mut Window) {
    let elapsed = self.last_render_start.elapsed();
    self.last_render_start = Instant::now();

    self.point_cloud_renderer.clear();
    let frame = self.freenect_state.get_frame().unwrap();

    let start = Instant::now();
    let width = frame.width() as f32;
    let height = frame.height() as f32;

    for (i, row) in frame.iter().enumerate() {
      for (j, value) in row.enumerate() {
        let z = value.expect_float();
        if z <= 0.0 {
          continue;
        }

        let x = j as f32 / width - 0.5;
        let y = 0.5 - i as f32 / height;

        let color = Hsl::from(z * Z_HUE_SCALE, 100.0, 50.0);

        self.point_cloud_renderer.push(
          Point3::new(x * SCALE, y * SCALE, z * FINAL_Z_SCALE),
          Point3::new(
            color.get_red() / 255.0,
            color.get_green() / 255.0,
            color.get_blue() / 255.0,
          ),
        );
      }
    }

    let processing = start.elapsed();
    let num_points_text = format!(
      "{} FPS, processing: {processing:?}",
      1_000_000 / elapsed.as_micros()
    );

    window.draw_text(
      &num_points_text,
      &Point2::new(10.0, 20.0),
      60.0,
      &Font::default(),
      &Point3::new(1.0, 1.0, 1.0),
    );
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

fn main() {
  let freenect_state = FreenectState::new().unwrap();

  let window = Window::new("Kinect point cloud");
  let app = AppState::new(freenect_state).unwrap();

  window.render_loop(app);
}

/// Structure which manages the display of long-living points.
struct PointCloudRenderer {
  shader: Effect,
  pos: ShaderAttribute<Point3<f32>>,
  color: ShaderAttribute<Point3<f32>>,
  proj: ShaderUniform<Matrix4<f32>>,
  view: ShaderUniform<Matrix4<f32>>,
  colored_points: GPUVec<Point3<f32>>,
  point_size: f32,
}

impl PointCloudRenderer {
  /// Creates a new points renderer.
  fn new(point_size: f32) -> PointCloudRenderer {
    let mut shader = Effect::new_from_str(VERTEX_SHADER_SRC, FRAGMENT_SHADER_SRC);

    shader.use_program();

    PointCloudRenderer {
      colored_points: GPUVec::new(Vec::new(), BufferType::Array, AllocationType::StreamDraw),
      pos: shader.get_attrib::<Point3<f32>>("position").unwrap(),
      color: shader.get_attrib::<Point3<f32>>("color").unwrap(),
      proj: shader.get_uniform::<Matrix4<f32>>("proj").unwrap(),
      view: shader.get_uniform::<Matrix4<f32>>("view").unwrap(),
      shader,
      point_size,
    }
  }

  fn push(&mut self, point: Point3<f32>, color: Point3<f32>) {
    if let Some(colored_points) = self.colored_points.data_mut() {
      colored_points.push(point);
      colored_points.push(color);
    }
  }

  fn clear(&mut self) {
    if let Some(colored_points) = self.colored_points.data_mut() {
      colored_points.clear();
    }
  }
}

impl Renderer for PointCloudRenderer {
  /// Actually draws the points.
  fn render(&mut self, pass: usize, camera: &mut dyn Camera) {
    if self.colored_points.len() == 0 {
      return;
    }

    self.shader.use_program();
    self.pos.enable();
    self.color.enable();

    camera.upload(pass, &mut self.proj, &mut self.view);

    self.color.bind_sub_buffer(&mut self.colored_points, 1, 1);
    self.pos.bind_sub_buffer(&mut self.colored_points, 1, 0);

    let ctxt = Context::get();
    ctxt.point_size(self.point_size);
    ctxt.draw_arrays(Context::POINTS, 0, (self.colored_points.len() / 2) as i32);

    self.pos.disable();
    self.color.disable();
  }
}

const VERTEX_SHADER_SRC: &str = "#version 100
    attribute vec3 position;
    attribute vec3 color;
    varying   vec3 Color;
    uniform   mat4 proj;
    uniform   mat4 view;
    void main() {
        gl_Position = proj * view * vec4(position, 1.0);
        Color = color;
    }";

const FRAGMENT_SHADER_SRC: &str = "#version 100
#ifdef GL_FRAGMENT_PRECISION_HIGH
   precision highp float;
#else
   precision mediump float;
#endif

    varying vec3 Color;
    void main() {
        gl_FragColor = vec4(Color, 1.0);
    }";
