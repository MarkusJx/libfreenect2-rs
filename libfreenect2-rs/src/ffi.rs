#![allow(unused)]
#![allow(clippy::needless_lifetimes)]

use std::panic::UnwindSafe;

pub(crate) struct CallContext<'a> {
  pub(crate) func: Box<
    dyn Fn(
        crate::types::frame_type::FrameType,
        crate::types::frame::Frame<'static>,
      ) -> anyhow::Result<()>
      + 'a,
  >,
}

#[cxx::bridge]
pub(crate) mod libfreenect2 {
  pub enum FrameType {
    Color = 1,
    Ir = 2,
    Depth = 4,
  }

  #[derive(Debug)]
  pub enum FrameFormat {
    /// Invalid format.
    Invalid = 0,
    /// Raw bitstream. 'bytes_per_pixel' defines the number of bytes
    Raw = 1,
    /// A 4-byte float per pixel
    Float = 2,
    /// 4 bytes of B, G, R, and unused per pixel
    BGRX = 4,
    /// 4 bytes of R, G, B, and unused per pixel
    RGBX = 5,
    /// 1 byte of gray per pixel
    Gray = 6,
  }

  pub enum LogLevel {
    /// No logging.
    None = 0,
    /// Log errors only.
    Error = 1,
    /// Log warnings and errors.
    Warning = 2,
    /// Log info messages, warnings, and errors.
    Info = 3,
    /// Log debug messages, info messages, warnings, and errors.
    Debug = 4,
  }

  /// Kinect LED settings
  pub struct LedSettings {
    /// The ID of the LED. Must be either 0 or 1.
    id: u16,
    /// The LED mode. Can be either `Constant` or `Blink`.
    mode: LedMode,
    /// The LED intensity at the start of the interval.
    /// Must be in the range \[0, 1000].
    start_level: u16,
    /// The LED intensity at the end of the interval.
    /// Must be in the range \[0, 1000].
    /// Does not need to be greater than `start_level`.
    stop_level: u16,
    /// Blink interval in milliseconds.
    /// Only used if `mode` is `Blink`.
    /// Must be greater than 0.
    interval_ms: u16,
  }

  /// LED mode
  pub enum LedMode {
    /// Constant LED intensity.
    Constant = 0,
    /// Blinking LED.
    Blink = 1,
  }

  /// Packet pipeline
  pub enum PacketPipeline {
    /// CPU packet pipeline.
    /// This is the default pipeline.
    CPU = 0,
    #[cfg(feature = "opengl")]
    /// OpenGL packet pipeline.
    /// Requires the `opengl` feature.
    OpenGL = 1,
    #[cfg(feature = "opencl")]
    /// OpenCL packet pipeline.
    /// Requires the `opencl` feature.
    OpenCL = 2,
    #[cfg(feature = "opencl")]
    /// OpenCL packet pipeline with the phase
    /// unwrapping algorithm described in
    /// http://www.cvl.isy.liu.se/research/datasets/kinect2-dataset/.
    /// Requires the `opencl` feature.
    OpenCLKDE = 3,
  }

  extern "Rust" {
    type CallContext<'a>;
  }

  #[namespace = "libfreenect2"]
  unsafe extern "C++" {
    include!("libfreenect2/frame_listener.hpp");
    include!("libfreenect2/led_settings.h");

    pub type FrameListener<'a>;
  }

  #[namespace = "libfreenect2_ffi"]
  unsafe extern "C++" {
    include!("frame.hpp");
    include!("libfreenect2.hpp");
    include!("registration.hpp");
    include!("freenect2_device.hpp");
    include!("config.hpp");
    include!("logger.hpp");

    fn create_frame_listener<'a>(
      ctx: Box<CallContext<'a>>,
      on_new_frame: fn(FrameType, UniquePtr<Frame<'static>>, &Box<CallContext<'a>>) -> String,
    ) -> Result<UniquePtr<FrameListener<'a>>>;

    pub type Freenect2;

    fn enumerate_devices(self: Pin<&mut Freenect2>) -> Result<i32>;
    fn get_device_serial_number(self: Pin<&mut Freenect2>, idx: i32) -> Result<String>;
    fn get_default_device_serial_number(self: Pin<&mut Freenect2>) -> Result<String>;

    unsafe fn open_device_by_id<'a>(
      self: Pin<&mut Freenect2>,
      idx: i32,
    ) -> Result<UniquePtr<Freenect2Device<'a>>>;
    unsafe fn open_device_by_id_with_packet_pipeline<'a>(
      self: Pin<&mut Freenect2>,
      idx: i32,
      pipeline: PacketPipeline,
    ) -> Result<UniquePtr<Freenect2Device<'a>>>;

    unsafe fn open_device_by_serial<'a>(
      self: Pin<&mut Freenect2>,
      serial: &str,
    ) -> Result<UniquePtr<Freenect2Device<'a>>>;
    unsafe fn open_device_by_serial_with_packet_pipeline<'a>(
      self: Pin<&mut Freenect2>,
      serial: &str,
      pipeline: PacketPipeline,
    ) -> Result<UniquePtr<Freenect2Device<'a>>>;

    unsafe fn open_default_device<'a>(
      self: Pin<&mut Freenect2>,
    ) -> Result<UniquePtr<Freenect2Device<'a>>>;
    unsafe fn open_default_device_with_packet_pipeline<'a>(
      self: Pin<&mut Freenect2>,
      pipeline: PacketPipeline,
    ) -> Result<UniquePtr<Freenect2Device<'a>>>;

    /// Create a new Freenect2 instance.
    ///
    /// # Safety
    /// No Freenect2 instance should be created
    /// while another one is still alive.
    unsafe fn create_freenect2() -> Result<UniquePtr<Freenect2>>;

    pub type Freenect2Device<'a>;

    unsafe fn get_serial_number(self: Pin<&mut Freenect2Device>) -> Result<String>;
    unsafe fn get_firmware_version(self: Pin<&mut Freenect2Device>) -> Result<String>;
    unsafe fn get_registration(self: Pin<&mut Freenect2Device>) -> Result<UniquePtr<Registration>>;

    unsafe fn start(self: Pin<&mut Freenect2Device>) -> Result<bool>;
    unsafe fn start_streams(
      self: Pin<&mut Freenect2Device>,
      color: bool,
      depth: bool,
    ) -> Result<bool>;
    unsafe fn stop(self: Pin<&mut Freenect2Device>) -> Result<bool>;
    unsafe fn close(self: Pin<&mut Freenect2Device>) -> Result<bool>;

    unsafe fn set_color_frame_listener<'a>(
      self: Pin<&mut Freenect2Device<'a>>,
      listener: &'a UniquePtr<FrameListener>,
    ) -> Result<()>;
    unsafe fn set_ir_and_depth_frame_listener<'a>(
      self: Pin<&mut Freenect2Device<'a>>,
      listener: &'a UniquePtr<FrameListener>,
    ) -> Result<()>;

    unsafe fn set_config<'a>(
      self: Pin<&mut Freenect2Device<'a>>,
      config: &'a UniquePtr<Config>,
    ) -> Result<()>;

    unsafe fn set_led_settings(
      self: Pin<&mut Freenect2Device>,
      settings: &LedSettings,
    ) -> Result<()>;

    pub type Frame<'a>;

    unsafe fn width(self: &Frame) -> u64;
    unsafe fn height(self: &Frame) -> u64;
    unsafe fn bytes_per_pixel(self: &Frame) -> u64;
    unsafe fn data(self: &Frame) -> *mut u8;
    unsafe fn timestamp(self: &Frame) -> u32;
    unsafe fn sequence(self: &Frame) -> u32;
    unsafe fn exposure(self: &Frame) -> f32;
    unsafe fn gain(self: &Frame) -> f32;
    unsafe fn gamma(self: &Frame) -> f32;
    unsafe fn status(self: &Frame) -> u32;
    unsafe fn format(self: &Frame) -> FrameFormat;

    #[allow(clippy::too_many_arguments)]
    unsafe fn create_frame(
      width: u64,
      height: u64,
      bytes_per_pixel: u64,
      data: *mut u8,
      timestamp: u32,
      sequence: u32,
      exposure: f32,
      gain: f32,
      gamma: f32,
      status: u32,
      format: FrameFormat,
    ) -> UniquePtr<Frame<'static>>;

    pub type Config;

    fn get_min_depth(self: &Config) -> f32;
    fn get_max_depth(self: &Config) -> f32;
    fn get_enable_bilateral_filter(self: &Config) -> bool;
    fn get_enable_edge_aware_filter(self: &Config) -> bool;
    fn set_min_depth(self: Pin<&mut Config>, min_depth: f32);
    fn set_max_depth(self: Pin<&mut Config>, max_depth: f32);
    fn set_enable_bilateral_filter(self: Pin<&mut Config>, enable: bool);
    fn set_enable_edge_aware_filter(self: Pin<&mut Config>, enable: bool);

    fn create_config() -> Result<UniquePtr<Config>>;

    pub type Registration;
    unsafe fn map_depth_to_color(
      self: &Registration,
      depth: &Frame,
      color: &Frame,
      undistorted_depth: Pin<&mut Frame>,
      color_depth_image: Pin<&mut Frame>,
      enable_filter: bool,
    ) -> Result<()>;
    unsafe fn map_depth_to_full_color(
      self: &Registration,
      depth: &Frame,
      color: &Frame,
      undistorted_depth: Pin<&mut Frame>,
      color_depth_image: Pin<&mut Frame>,
      enable_filter: bool,
      big_depth: Pin<&mut Frame>,
    ) -> Result<()>;
    unsafe fn undistort_depth(
      self: &Registration,
      depth: &Frame,
      undistorted_depth: Pin<&mut Frame>,
    ) -> Result<()>;

    fn create_logger(log_fn: fn(LogLevel, &String)) -> Result<()>;
  }

  #[cfg(debug_assertions)]
  #[namespace = "libfreenect2_ffi::test"]
  unsafe extern "C++" {
    unsafe fn call_frame_listener<'a>(
      listener: &mut UniquePtr<FrameListener<'a>>,
      frame_type: FrameType,
      width: u64,
      height: u64,
      bytes_per_pixel: u64,
      data: *mut u8,
    ) -> Result<()>;
  }
}
