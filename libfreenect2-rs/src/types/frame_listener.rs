use cxx::UniquePtr;

use crate::ffi;
use crate::types::frame::Frame;
use crate::types::frame_type::FrameType;

pub struct FrameListener<'a>(pub(crate) UniquePtr<ffi::libfreenect2::FrameListener<'a>>);

impl<'a> FrameListener<'a> {
  pub fn new<F: Fn(FrameType, Frame) + 'a>(f: F) -> anyhow::Result<Self> {
    let ctx = Box::new(ffi::CallContext { func: Box::new(f) });

    ffi::libfreenect2::create_frame_listener(ctx, |frame_type, frame, ctx| {
      let ctx = ctx.as_ref();
      let func = &ctx.func;

      func(frame_type.into(), Frame::new(frame));
    })
    .map(Self)
    .map_err(Into::into)
  }
}

unsafe impl<'a> Send for FrameListener<'a> {}
unsafe impl<'a> Sync for FrameListener<'a> {}
