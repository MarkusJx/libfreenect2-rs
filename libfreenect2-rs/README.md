# libfreenect2-rs

rust bindings for [libfreenect2](https://github.com/OpenKinect/libfreenect2).

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
libfreenect2 = { version = "0.1", git = "https://github.com/MarkusJx/libfreenect2-rs" }
```

## Example

```rust
use libfreenect2::freenect2::Freenect2;
use libfreenect2_rs::frame_listener::FrameListener;
use libfreenect2_rs::frame::Freenect2Frame;
use libfreenect2_rs::config::Config;

fn main() -> anyhow::Result<()> {
    let mut freenect2 = Freenect2::new();
    let num_devices = freenect2.enumerate_devices();
    println!("Number of devices: {}", num_devices);

    let mut device = freenect2.open_default_device()?;

    let frame_listener = FrameListener::new(|frame_type, frame| {
        println!("Frame type: {:?}", frame_type);
        println!("Frame size: {}x{}", frame.width(), frame.height());
    })?;

    device.set_ir_and_depth_frame_listener(frame_listener)?;
    device.start_streams(true, true)?;

    let mut config = Config::new()?;
    config.set_max_depth(5.0);
    device.set_config(config)?;
}
```