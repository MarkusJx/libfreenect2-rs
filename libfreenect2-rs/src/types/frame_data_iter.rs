use crate::frame_data::FrameDataValue;
use crate::types::frame::Freenect2Frame;

/// An iterator over the rows in a frame.
/// Each row is an iterator over the values in that row.
/// The values are in the format specified by the frame.
/// The values are in row-major order.
/// The actual values can be accessed by iterating over the
/// returned [`FrameRowIter`] iterator.
pub struct FrameDataIter<'a, T: FrameDataValue<'a>> {
  data: &'a T,
  frame: &'a dyn Freenect2Frame,
  row: usize,
}

impl<'a, T: FrameDataValue<'a>> FrameDataIter<'a, T> {
  pub(crate) fn new(frame: &'a dyn Freenect2Frame, data: &'a T) -> Self {
    Self {
      frame,
      data,
      row: 0,
    }
  }
}

/// An iterator over the values in a row of a frame.
/// The values are in the format specified by the frame.
pub struct FrameRowIter<'a, T: FrameDataValue<'a>> {
  data: &'a T,
  frame: &'a dyn Freenect2Frame,
  row: usize,
  col: usize,
}

impl<'a, T: FrameDataValue<'a>> Iterator for FrameDataIter<'a, T> {
  type Item = FrameRowIter<'a, T>;

  fn next(&mut self) -> Option<Self::Item> {
    if self.row >= self.frame.height() {
      return None;
    }

    let row = self.row;
    self.row += 1;

    Some(FrameRowIter {
      data: self.data,
      frame: self.frame,
      row,
      col: 0,
    })
  }
}

impl<'a, T: FrameDataValue<'a>> Iterator for FrameRowIter<'a, T> {
  type Item = T::Value;

  fn next(&mut self) -> Option<Self::Item> {
    if self.col >= self.frame.width() {
      return None;
    }

    let col = self.col;
    self.col += 1;

    Some(self.data.get_pixel(col, self.row))
  }
}
