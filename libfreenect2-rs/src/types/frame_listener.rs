use crate::ffi;
use crate::ffi::CallContext;
use crate::frame::OwnedFrame;
use crate::types::frame::Frame;
use crate::types::frame_type::FrameType;
use anyhow::anyhow;
use cxx::UniquePtr;
use std::panic::{catch_unwind, UnwindSafe};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};

/// A trait for types that can be converted into a [`FrameListener`].
pub trait AsFrameListener<'a> {
  /// Convert the value into a [`FrameListener`].
  fn as_frame_listener(&self) -> &FrameListener<'a>;
}

/// A listener for new frames.
/// [`MultiFrameListener`] may be used instead if
/// you need to listen for multiple frame types at once.
///
/// Must be passed to [`crate::freenect2_device::Freenect2Device::set_color_frame_listener`]
/// or [`crate::freenect2_device::Freenect2Device::set_ir_and_depth_frame_listener`]
/// to receive frames.
pub struct FrameListener<'a>(pub(crate) UniquePtr<ffi::libfreenect2::FrameListener<'a>>);

impl<'a> FrameListener<'a> {
  /// Create a new [`FrameListener`] with a closure that will be called
  /// when a new frame is received.
  /// If the closure panics, the panic will be caught and logged.
  ///
  /// # Arguments
  /// * `f` - The closure to call when a new frame is received.
  ///
  /// # Example
  /// ```
  /// use libfreenect2_rs::frame_listener::FrameListener;
  ///
  /// let listener = FrameListener::new(|ty, frame| {
  ///   println!("Received frame of type {:?}", ty);
  ///   Ok(())
  /// }).unwrap();
  /// ```
  pub fn new<F: Fn(FrameType, Frame<'static>) -> anyhow::Result<()> + UnwindSafe + Clone + 'a>(
    f: F,
  ) -> anyhow::Result<Self> {
    let ctx = Box::new(CallContext {
      func: Box::new(move |ty, frame| {
        let func = f.clone();

        catch_unwind(move || func(ty, frame)).unwrap_or_else(|e| {
          log::error!("Frame listener closure panicked: {:?}", e);
          Err(anyhow!("{:?}", e))
        })
      }),
    });

    Self::create_self(ctx)
  }

  /// Create a new [`FrameListener`] with a closure that will be called
  /// when a new frame is received.
  /// If the closure panics, the program will abort.
  /// A backtrace may not be available in this case.
  ///
  /// # Arguments
  /// * `f` - The closure to call when a new frame is received.
  ///
  /// # Safety
  /// This method is unsafe because it may cause the program to abort if the closure panics.
  /// The closure must not panic. Otherwise, undefined behavior may occur.
  pub unsafe fn new_no_panic<F: Fn(FrameType, Frame) -> anyhow::Result<()> + 'a>(
    f: F,
  ) -> anyhow::Result<Self> {
    let ctx = Box::new(CallContext { func: Box::new(f) });

    Self::create_self(ctx)
  }

  fn create_self(ctx: Box<CallContext<'a>>) -> anyhow::Result<Self> {
    ffi::libfreenect2::create_frame_listener(ctx, |frame_type, frame, ctx| {
      let ctx = ctx.as_ref();
      let func = ctx.func.as_ref();

      match func(frame_type.into(), Frame::new(frame)) {
        Err(e) => format!("{:?}", e),
        Ok(_) => "".to_string(),
      }
    })
    .map(Self)
    .map_err(Into::into)
  }
}

impl<'a> AsFrameListener<'a> for FrameListener<'a> {
  fn as_frame_listener(&self) -> &FrameListener<'a> {
    self
  }
}

unsafe impl<'a> Send for FrameListener<'a> {}
unsafe impl<'a> Sync for FrameListener<'a> {}

/// A map of frames received by a [`MultiFrameListener`].+
/// The frame types stored are the types that the listener was created with.
pub struct FrameMap<T: From<Frame<'static>>> {
  color: Option<T>,
  ir: Option<T>,
  depth: Option<T>,
}

impl<T: From<Frame<'static>>> Default for FrameMap<T> {
  fn default() -> Self {
    Self {
      color: None,
      ir: None,
      depth: None,
    }
  }
}

impl<T: From<Frame<'static>>> FrameMap<T> {
  /// Insert a frame into the map.
  pub fn insert(&mut self, ty: FrameType, frame: T) {
    match ty {
      FrameType::Color => self.set_color(frame),
      FrameType::Ir => self.set_ir(frame),
      FrameType::Depth => self.set_depth(frame),
    }
  }

  /// Set the color frame.
  pub fn set_color<U: Into<T>>(&mut self, frame: U) {
    self.color = Some(frame.into());
  }

  /// Set the ir frame.
  pub fn set_ir<U: Into<T>>(&mut self, frame: U) {
    self.ir = Some(frame.into());
  }

  /// Set the depth frame.
  pub fn set_depth<U: Into<T>>(&mut self, frame: U) {
    self.depth = Some(frame.into());
  }

  /// Check if the map contains all the frame types.
  fn contains_values(&self, types: &FrameTypes) -> bool {
    (!types.color || self.color.is_some())
      && (!types.ir || self.ir.is_some())
      && (!types.depth || self.depth.is_some())
  }

  /// Get the color frame.
  pub fn color(&self) -> Option<&T> {
    self.color.as_ref()
  }

  /// Get the ir frame.
  pub fn ir(&self) -> Option<&T> {
    self.ir.as_ref()
  }

  /// Get the depth frame.
  pub fn depth(&self) -> Option<&T> {
    self.depth.as_ref()
  }

  /// Take the color frame.
  pub fn take_color(&mut self) -> Option<T> {
    self.color.take()
  }

  /// Take the ir frame.
  pub fn take_ir(&mut self) -> Option<T> {
    self.ir.take()
  }

  /// Take the depth frame.
  pub fn take_depth(&mut self) -> Option<T> {
    self.depth.take()
  }

  /// Expect the color frame.
  /// If the frame is not found, an error is returned.
  pub fn expect_color(&self) -> anyhow::Result<&T> {
    self
      .color
      .as_ref()
      .ok_or_else(|| anyhow!("Color frame not found"))
  }

  /// Expect the ir frame.
  /// If the frame is not found, an error is returned.
  pub fn expect_ir(&self) -> anyhow::Result<&T> {
    self
      .ir
      .as_ref()
      .ok_or_else(|| anyhow!("Ir frame not found"))
  }

  /// Expect the depth frame.
  /// If the frame is not found, an error is returned.
  pub fn expect_depth(&self) -> anyhow::Result<&T> {
    self
      .depth
      .as_ref()
      .ok_or_else(|| anyhow!("Depth frame not found"))
  }

  /// Take the color frame.
  /// If the frame is not found, an error is returned.
  pub fn expect_take_color(&mut self) -> anyhow::Result<T> {
    self
      .color
      .take()
      .ok_or_else(|| anyhow!("Color frame not found"))
  }

  /// Take the ir frame.
  /// If the frame is not found, an error is returned.
  pub fn expect_take_ir(&mut self) -> anyhow::Result<T> {
    self.ir.take().ok_or_else(|| anyhow!("Ir frame not found"))
  }

  /// Take the depth frame.
  /// If the frame is not found, an error is returned.
  pub fn expect_take_depth(&mut self) -> anyhow::Result<T> {
    self
      .depth
      .take()
      .ok_or_else(|| anyhow!("Depth frame not found"))
  }
}

#[derive(Clone)]
struct FrameTypes {
  color: bool,
  ir: bool,
  depth: bool,
}

impl FrameTypes {
  fn new(types: &[FrameType]) -> Self {
    Self {
      color: types.contains(&FrameType::Color),
      ir: types.contains(&FrameType::Ir),
      depth: types.contains(&FrameType::Depth),
    }
  }
}

/// A [`MultiFrameListener`] that returns [`OwnedFrame`]s.
pub type OwnedFramesMultiFrameListener<'a> = MultiFrameListener<'a, OwnedFrame>;
/// A [`MultiFrameListener`] that returns [`Frame`]s.
pub type NativeFramesMultiFrameListener<'a> = MultiFrameListener<'a, Frame<'static>>;

/// A listener for multiple frame types.
/// This listener will wait for all frame types to be received before returning the frames.
/// If you need to listen for the frames individually, use [`FrameListener`] instead.
pub struct MultiFrameListener<'a, T: From<Frame<'static>> + Send + Sync> {
  listener: FrameListener<'a>,
  rx: Mutex<Receiver<FrameMap<T>>>,
}

impl<'a, T: From<Frame<'static>> + Send + Sync + 'static> MultiFrameListener<'a, T> {
  /// Create a new [`MultiFrameListener`] that listens for the specified frame types.
  /// The listener will wait for all frame types to be received before returning the frames.
  ///
  /// # Arguments
  /// * `frame_types` - The frame types to listen for. Must contain at least one element.
  ///
  /// # Errors
  /// Returns an error if no frame types are specified
  /// or the underlying frame listener could not be created.
  ///
  /// # Example
  /// ```no_run
  /// use libfreenect2_rs::frame_listener::OwnedFramesMultiFrameListener;
  /// use libfreenect2_rs::frame_type::FrameType;
  ///
  /// let listener = OwnedFramesMultiFrameListener::new(&[FrameType::Color, FrameType::Depth]).unwrap();
  ///
  /// /// Set the listener and start the device
  ///
  /// let frames = listener.get_frames().unwrap();
  /// ```
  pub fn new(frame_types: &[FrameType]) -> anyhow::Result<Self> {
    anyhow::ensure!(
      !frame_types.is_empty(),
      "At least one frame type must be specified"
    );

    let frames = Arc::new(Mutex::new(FrameMap::default()));
    let types = FrameTypes::new(frame_types);
    let (tx, rx) = channel();

    Ok(Self {
      listener: FrameListener::new(move |ty, frame| {
        let mut frames = frames
          .lock()
          .map_err(|e| anyhow::anyhow!("Failed to lock frame map: {e}"))?;
        frames.insert(ty, T::from(frame));

        if frames.contains_values(&types) {
          let old_frames = std::mem::take(&mut *frames);
          tx.send(old_frames)?;
        }

        Ok(())
      })?,
      rx: Mutex::new(rx),
    })
  }

  /// Get the next set of frames.
  /// This will block until all frame types have been received.
  /// If you need to wait with a timeout, use [`Self::get_frames_with_timeout`] instead.
  pub fn get_frames(&self) -> anyhow::Result<FrameMap<T>> {
    let rx = self
      .rx
      .lock()
      .map_err(|_| anyhow!("Failed to lock receiver"))?;
    rx.recv().map_err(Into::into)
  }

  /// Get the next set of frames with a timeout.
  /// If the frames are not received within the timeout, an error is returned.
  /// If you need to wait indefinitely, use [`Self::get_frames`] instead.
  ///
  /// # Arguments
  /// * `timeout` - The maximum amount of time to wait for the frames.
  ///
  /// # Errors
  /// Returns an error if the frames are not received within the timeout.
  pub fn get_frames_with_timeout(
    &self,
    timeout: std::time::Duration,
  ) -> anyhow::Result<FrameMap<T>> {
    let rx = self
      .rx
      .lock()
      .map_err(|_| anyhow!("Failed to lock receiver"))?;
    rx.recv_timeout(timeout).map_err(Into::into)
  }
}

impl<'a, T: From<Frame<'static>> + Send + Sync> AsFrameListener<'a> for MultiFrameListener<'a, T> {
  fn as_frame_listener(&self) -> &FrameListener<'a> {
    &self.listener
  }
}
