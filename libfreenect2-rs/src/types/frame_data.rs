pub type FrameMatrix<T> = Vec<Vec<T>>;

#[derive(Debug)]
pub struct RGBX {
  pub r: u8,
  pub g: u8,
  pub b: u8,
  pub x: u8,
}

pub enum FrameData {
  Raw(FrameMatrix<Vec<u8>>),
  Float(FrameMatrix<f32>),
  RGBX(FrameMatrix<RGBX>),
  Gray(FrameMatrix<u8>),
}
