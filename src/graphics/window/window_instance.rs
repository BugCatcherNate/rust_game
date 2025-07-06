use std::sync::Arc;
use winit::event::{Event, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::window::{Window, WindowBuilder};

pub struct WindowBundle {
    pub window: Arc<Window>,
    pub event_loop: EventLoop<()>,
}


pub fn create_window_bundle() -> WindowBundle {


    let event_loop = EventLoop::new();

    let window = Arc::new(WindowBuilder::new().build(&event_loop).unwrap());

    WindowBundle{window, event_loop}

}




