#[cfg(debug_assertions)]
use crate::ffi::libfreenect2::{call_frame_listener, FrameType};
#[cfg(debug_assertions)]
use crate::types::frame::Freenect2Frame;
use crate::types::frame_listener::FrameListener;

#[test]
fn test_create_frame_listener() {
  FrameListener::new(|_type, _frame| Ok(())).unwrap();
}

#[test]
#[cfg(debug_assertions)]
fn test_call_frame_listener() {
  let mut listener = FrameListener::new(|ty, frame| {
    assert_eq!(ty, FrameType::Color.into());
    assert_eq!(frame.width(), 1);
    assert_eq!(frame.height(), 2);
    assert_eq!(frame.bytes_per_pixel(), 2);
    assert_eq!(frame.raw_data(), [1, 2, 3, 4]);

    Ok(())
  })
  .unwrap();

  let mut data = vec![1, 2, 3, 4];
  unsafe {
    call_frame_listener(
      &mut listener.0,
      FrameType::Color,
      1,
      2,
      2,
      data.as_mut_ptr(),
    )
    .unwrap();
  }
}

#[test]
#[cfg(debug_assertions)]
fn test_call_throwing_frame_listener() {
  let mut listener = FrameListener::new(|_, _| anyhow::bail!("Test")).unwrap();

  let mut data = vec![1, 2, 3, 4];
  let res = unsafe {
    call_frame_listener(
      &mut listener.0,
      FrameType::Color,
      1,
      2,
      2,
      data.as_mut_ptr(),
    )
  };

  assert!(res.is_err());
  assert!(res.unwrap_err().to_string().contains("Test"))
}
