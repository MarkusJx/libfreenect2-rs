mod constants;
mod freenect_state;
mod renderer;
mod state;

extern crate kiss3d;
extern crate nalgebra as na;

use crate::freenect_state::FreenectState;
use crate::state::AppState;
use clap::{Parser, ValueEnum};
use kiss3d::window::Window;

#[derive(ValueEnum, Clone, Eq, PartialEq)]
enum RenderType {
  Depth,
  Color,
  FullColor,
}

impl RenderType {
  fn is_color(&self) -> bool {
    match self {
      RenderType::Color | RenderType::FullColor => true,
      RenderType::Depth => false,
    }
  }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct CommendLineArgs {
  #[arg(short, long)]
  render_type: Option<RenderType>,
}

fn main() -> anyhow::Result<()> {
  let args = CommendLineArgs::parse();
  let render_type = args.render_type.unwrap_or(RenderType::Depth);

  let freenect_state = FreenectState::new(render_type.clone())?;
  let window = Window::new("Kinect point cloud");
  let app = AppState::new(freenect_state, render_type)?;

  window.render_loop(app);
  Ok(())
}
