use crate::types::freenect2::Freenect2;
use std::time::Duration;

#[test]
pub fn test_create_freenect2() {
  assert!(Freenect2::new().is_ok());
}

#[test]
fn test_enumerate_devices() {
  let mut freenect2 = Freenect2::new().unwrap();
  assert!(freenect2.enumerate_devices().is_ok());
}

#[test]
fn test_multiple() {
  {
    let mut first = Freenect2::new().unwrap();
    let mut second = Freenect2::new().unwrap();

    assert!(first.enumerate_devices().is_ok());
    assert!(second.enumerate_devices().is_ok());

    drop(first);
    assert!(second.enumerate_devices().is_ok());

    let mut third = Freenect2::new().unwrap();
    assert!(third.enumerate_devices().is_ok());
  }

  for _ in 0..10 {
    std::thread::sleep(Duration::from_millis(100));
    if !Freenect2::has_instance() {
      break;
    }
  }
  assert!(!Freenect2::has_instance());

  let mut fourth = Freenect2::new().unwrap();
  assert!(fourth.enumerate_devices().is_ok());
}
