use anyhow::Result;
use state::State;
use wgpu::SurfaceError;
use winit::{event::*, event_loop::*, window::WindowBuilder};

mod character;
mod character_buffer;
mod globals;
mod shader_param;
mod state;
mod texture;
mod vertex;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("miniterm 1999")
        .build(&event_loop)?;

    let mut state = State::new(&window).await?;

    event_loop.run(move |event, _, control_flow| match event {
        Event::RedrawRequested(window_id) if window_id == window.id() => match state.render() {
            Ok(_) => {}
            Err(SurfaceError::Lost) => state.resize(state.size()),
            Err(SurfaceError::OutOfMemory) => {
                eprintln!("Out of memory, exitting");
                *control_flow = ControlFlow::Exit;
            }
            Err(error) => eprintln!("{}", error),
        },
        Event::WindowEvent { window_id, event } if window_id == window.id() => {
            match &event {
                WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                WindowEvent::Resized(new_size) => state.resize(*new_size),
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    state.resize(**new_inner_size)
                }
                _ => {}
            }

            if state.input(event) {
                window.request_redraw();
            }
        }
        Event::MainEventsCleared => {
            state.update();
            window.request_redraw();
        }
        _ => {}
    });
}
