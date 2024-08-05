pub const MAX_DEPTH: f32 = 6.5;
pub const SCALE: f32 = 10.0;
pub const Z_SCALE: f32 = 3.0;

pub const FINAL_Z_SCALE: f32 = Z_SCALE / (MAX_DEPTH * 1000.0) * SCALE;
pub const Z_HUE_SCALE: f32 = 255.0 / (MAX_DEPTH * 1000.0);
