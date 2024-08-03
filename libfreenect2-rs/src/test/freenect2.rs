use crate::types::freenect2::Freenect2;

#[test]
pub fn test_create_freenect2() {
  assert!(Freenect2::new().is_ok());
}

#[test]
fn test_enumerate_devices() {
  let mut freenect2 = Freenect2::new().unwrap();
  assert!(freenect2.enumerate_devices().is_ok());
}
