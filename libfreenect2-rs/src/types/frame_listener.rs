use crate::ffi;
use crate::ffi::CallContext;
use crate::types::frame::Frame;
use crate::types::frame_type::FrameType;
use anyhow::anyhow;
use cxx::UniquePtr;
use std::collections::HashSet;
use std::marker::PhantomData;
#[cfg(feature = "catch-frame-listener-panic")]
use std::panic::{catch_unwind, UnwindSafe};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};

pub trait AsFrameListener<'a> {
  fn as_frame_listener(&'a self) -> &'a FrameListener<'a>;
}

pub struct FrameListener<'a>(pub(crate) UniquePtr<ffi::libfreenect2::FrameListener<'a>>);

impl<'a> FrameListener<'a> {
  #[cfg(feature = "catch-frame-listener-panic")]
  pub fn new<F: Fn(FrameType, Frame<'a>) + UnwindSafe + Clone + 'a>(f: F) -> anyhow::Result<Self> {
    let ctx = Box::new(CallContext {
      func: Box::new(move |ty, frame| {
        let func = f.clone();
        if let Err(e) = catch_unwind(move || func(ty, frame)) {
          log::error!("Panic in frame listener: {:?}", e);
        }
      }),
    });

    Self::create_self(ctx)
  }

  #[cfg(not(feature = "catch-frame-listener-panic"))]
  pub fn new<F: Fn(FrameType, Frame) + 'a>(f: F) -> anyhow::Result<Self> {
    #[cfg(not(feature = "catch-frame-listener-panic"))]
    let ctx = Box::new(ffi::CallContext { func: Box::new(f) });

    Self::create_self(ctx)
  }

  fn create_self(ctx: Box<CallContext<'a>>) -> anyhow::Result<Self> {
    ffi::libfreenect2::create_frame_listener(ctx, |frame_type, frame, ctx| {
      let ctx = ctx.as_ref();
      let func = ctx.func.as_ref();

      func(frame_type.into(), Frame::new(frame))
    })
    .map(Self)
    .map_err(Into::into)
  }
}

impl<'a> AsFrameListener<'a> for FrameListener<'a> {
  fn as_frame_listener(&'a self) -> &'a FrameListener<'a> {
    self
  }
}

unsafe impl<'a> Send for FrameListener<'a> {}
unsafe impl<'a> Sync for FrameListener<'a> {}

pub struct FrameMap<'a, T: From<Frame<'a>>> {
  color: Option<T>,
  ir: Option<T>,
  depth: Option<T>,
  _phantom: PhantomData<&'a ()>,
}

impl<'a, T: From<Frame<'a>>> Default for FrameMap<'a, T> {
  fn default() -> Self {
    Self {
      color: None,
      ir: None,
      depth: None,
      _phantom: PhantomData,
    }
  }
}

impl<'a, T: From<Frame<'a>>> FrameMap<'a, T> {
  pub fn insert(&mut self, ty: FrameType, frame: T) {
    match ty {
      FrameType::Color => self.set_color(frame),
      FrameType::Ir => self.set_ir(frame),
      FrameType::Depth => self.set_depth(frame),
    }
  }

  pub fn set_color(&mut self, frame: T) {
    self.color = Some(frame);
  }

  pub fn set_ir(&mut self, frame: T) {
    self.ir = Some(frame);
  }

  pub fn set_depth(&mut self, frame: T) {
    self.depth = Some(frame);
  }

  fn contains_values(&self, types: &FrameTypes) -> bool {
    (!types.color || self.color.is_some())
      && (!types.ir || self.ir.is_some())
      && (!types.depth || self.depth.is_some())
  }

  pub fn color(&self) -> Option<&T> {
    self.color.as_ref()
  }

  pub fn ir(&self) -> Option<&T> {
    self.ir.as_ref()
  }

  pub fn depth(&self) -> Option<&T> {
    self.depth.as_ref()
  }

  pub fn take_color(&mut self) -> Option<T> {
    self.color.take()
  }

  pub fn take_ir(&mut self) -> Option<T> {
    self.ir.take()
  }

  pub fn take_depth(&mut self) -> Option<T> {
    self.depth.take()
  }

  pub fn expect_color(&self) -> anyhow::Result<&T> {
    self
      .color
      .as_ref()
      .ok_or_else(|| anyhow!("Color frame not found"))
  }

  pub fn expect_ir(&self) -> anyhow::Result<&T> {
    self
      .ir
      .as_ref()
      .ok_or_else(|| anyhow!("Ir frame not found"))
  }

  pub fn expect_depth(&self) -> anyhow::Result<&T> {
    self
      .depth
      .as_ref()
      .ok_or_else(|| anyhow!("Depth frame not found"))
  }

  pub fn expect_take_color(&mut self) -> anyhow::Result<T> {
    self
      .color
      .take()
      .ok_or_else(|| anyhow!("Color frame not found"))
  }

  pub fn expect_take_ir(&mut self) -> anyhow::Result<T> {
    self.ir.take().ok_or_else(|| anyhow!("Ir frame not found"))
  }

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
  fn new(types: &HashSet<FrameType>) -> Self {
    Self {
      color: types.contains(&FrameType::Color),
      ir: types.contains(&FrameType::Ir),
      depth: types.contains(&FrameType::Depth),
    }
  }
}

pub struct MultiFrameListener<'a, T: From<Frame<'a>>> {
  listener: FrameListener<'a>,
  rx: Mutex<Receiver<FrameMap<'a, T>>>,
}

impl<'a, T: From<Frame<'a>> + 'a> MultiFrameListener<'a, T> {
  pub fn new(frame_types: &HashSet<FrameType>) -> anyhow::Result<Self> {
    let frames = Arc::new(Mutex::new(FrameMap::default()));
    let types = FrameTypes::new(frame_types);
    let (tx, rx) = channel();

    Ok(Self {
      listener: FrameListener::new(move |ty, frame| {
        let mut frames = frames.lock().unwrap();
        frames.insert(ty, T::from(frame));

        if frames.contains_values(&types) {
          let old_frames = std::mem::take(&mut *frames);
          tx.send(old_frames).unwrap();
        }
      })?,
      rx: Mutex::new(rx),
    })
  }

  pub fn get_frames(&self) -> anyhow::Result<FrameMap<'a, T>> {
    let rx = self
      .rx
      .lock()
      .map_err(|_| anyhow!("Failed to lock receiver"))?;
    rx.recv().map_err(Into::into)
  }

  pub fn get_frames_with_timeout(
    &self,
    timeout: std::time::Duration,
  ) -> anyhow::Result<FrameMap<'a, T>> {
    let rx = self
      .rx
      .lock()
      .map_err(|_| anyhow!("Failed to lock receiver"))?;
    rx.recv_timeout(timeout).map_err(Into::into)
  }
}

impl<'a, T: From<Frame<'a>>> AsFrameListener<'a> for MultiFrameListener<'a, T> {
  fn as_frame_listener(&'a self) -> &'a FrameListener<'a> {
    &self.listener
  }
}
