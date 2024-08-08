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
use log::LevelFilter;
use log4rs::append::console::ConsoleAppender;
use log4rs::config::{Appender, Root};

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

  fn depth_offset(&self) -> usize {
    match self {
      RenderType::FullColor => 1,
      _ => 0,
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
  log4rs::init_config(
    log4rs::Config::builder()
      .appender(Appender::builder().build("stdout", Box::new(ConsoleAppender::builder().build())))
      .build(Root::builder().appender("stdout").build(LevelFilter::Debug))?,
  )?;
  let args = CommendLineArgs::parse();
  let render_type = args.render_type.unwrap_or(RenderType::Depth);

  let freenect_state = FreenectState::new(render_type.clone())?;
  let window = Window::new("Kinect point cloud");
  let app = AppState::new(freenect_state, render_type)?;

  window.render_loop(app);
  Ok(())
}
