use std::{io::Write, process::Command};

use anyhow::Result;
use state::State;
use try_read::TryReader;
use wgpu::SurfaceError;
use winit::{
    event::*,
    event_loop::*,
    window::{Fullscreen, WindowBuilder},
};

mod character;
mod character_buffer;
mod globals;
mod shader_param;
mod state;
mod texture;
mod try_read;
mod vertex;

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let process = ptyprocess::PtyProcess::spawn(Command::new(std::env::var("SHELL")?))?;
    let reader = TryReader::new(process.get_pty_stream()?);

    let mut char_buffer = [0u8; 4];

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
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::F11),
                            ..
                        },
                    ..
                } => {
                    window.set_fullscreen(match window.fullscreen() {
                        Some(_) => None,
                        None => Some(Fullscreen::Borderless(None)),
                    });
                }
                WindowEvent::ReceivedCharacter(c) => {
                    // stream
                    //     .write_all(c.encode_utf8(&mut char_buffer).as_bytes())
                    //     .unwrap_or_else(|e| eprintln!("Could not write char to stdin of pty: {e}"));
                }
                _ => {}
            }

            if state.input(event) {
                window.request_redraw();
            }
        }
        Event::MainEventsCleared => {
            if let Some(str) = reader.try_read() {
                state.push_str(&str);
            }

            state.update();
            window.request_redraw();
        }
        _ => {}
    });
}
