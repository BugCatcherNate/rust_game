use winit::{
    event::*,
    event_loop::EventLoop,
    keyboard::{KeyCode, PhysicalKey},
    window::WindowBuilder,
};
use log::{warn, error};
use crate::graphics::state::State; 

pub async fn run() {
    let event_loop = EventLoop::new().unwrap();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut state = State::new(&window).await;
    event_loop.run(move |event, control_flow| match event {
        Event::WindowEvent {
            ref event,
            window_id,
        } if window_id == state.window.id() => match event {
            WindowEvent::CloseRequested
            | WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        state: ElementState::Pressed,
                        physical_key: PhysicalKey::Code(KeyCode::Escape),
                        ..
                    },
                ..
            } => control_flow.exit(),
            WindowEvent::RedrawRequested => {
                state.window().request_redraw();

                state.update();
                match state.render() {
                    Ok(_) => {}
                    Err(
                        wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated,
                    ) => state.resize(state.size),
                    Err(wgpu::SurfaceError::OutOfMemory) => {
                        error!("Out of Memory");
                        control_flow.exit();
                    }
                    Err(wgpu::SurfaceError::Timeout) => {
                        warn!("Surface timeout")            
                    }

                }
            },
            WindowEvent::Resized(physical_size) => {
                state.resize(*physical_size);
            },
            _ => {}
        },
        _ => {}
    });
}

