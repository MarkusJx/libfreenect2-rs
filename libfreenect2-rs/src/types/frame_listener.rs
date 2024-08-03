use cxx::UniquePtr;
#[cfg(feature = "catch-frame-listener-panic")]
use std::panic::{catch_unwind, UnwindSafe};

use crate::ffi;
use crate::ffi::CallContext;
use crate::types::frame::Frame;
use crate::types::frame_type::FrameType;

pub struct FrameListener<'a>(pub(crate) UniquePtr<ffi::libfreenect2::FrameListener<'a>>);

impl<'a> FrameListener<'a> {
  #[cfg(feature = "catch-frame-listener-panic")]
  pub fn new<F: Fn(FrameType, Frame) + UnwindSafe + Clone + 'a>(f: F) -> anyhow::Result<Self> {
    let ctx = Box::new(ffi::CallContext {
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

unsafe impl<'a> Send for FrameListener<'a> {}
unsafe impl<'a> Sync for FrameListener<'a> {}
