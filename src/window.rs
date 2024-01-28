use crate::input;
use winit::{
    event::*,
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
#[derive(Debug, Clone, Copy)]
pub struct WindowSize {
    pub width: u32,
    pub height: u32,
}
pub struct WindowInitSettings {
    pub on_update_fn: fn(),
    pub on_resize_func: fn(new_size: WindowSize),
    pub window_title: String,
    pub starting_size: WindowSize,
}

impl Default for WindowInitSettings {
    fn default() -> Self {
        let default_on_update_fn = || {};
        let default_on_resize_func = |_: WindowSize| {};
        Self {
            on_update_fn: default_on_update_fn,
            on_resize_func: default_on_resize_func,
            window_title: "Window Title".to_string(),
            starting_size: WindowSize {
                width: 128,
                height: 128,
            },
        }
    }
}

pub struct WinitWindow {
    on_update_fn: Option<fn()>,
    on_resize_func: Option<fn(new_size: WindowSize)>,
    event_loop: Option<EventLoop<()>>,
    window: Option<winit::window::Window>,
}
impl WinitWindow {
    fn new(settings: WindowInitSettings) -> Self {
        let event_loop = EventLoop::new();

        let window = WindowBuilder::new()
            .with_title(settings.window_title)
            .with_inner_size(winit::dpi::LogicalSize::new(
                settings.starting_size.width,
                settings.starting_size.height,
            ))
            .build(&event_loop)
            .unwrap();
        Self {
            on_update_fn: Some(settings.on_update_fn),
            on_resize_func: Some(settings.on_resize_func),
            event_loop: Some(event_loop),
            window: Some(window),
        }
    }
    pub fn get_window_size(&self) -> WindowSize {
        let size = self.window.as_ref().expect("Window not found").inner_size();
        WindowSize {
            width: size.width,
            height: size.height,
        }
    }
    pub fn get_winit_window(&self) -> Result<&winit::window::Window, String> {
        if let Some(win) = self.window.as_ref() {
            return Ok(win);
        }
        Err("No window found".to_string())
    }
    pub fn run(&mut self) {
        // moving data so that
        let event_loop: EventLoop<()> = std::mem::replace(&mut self.event_loop, None).unwrap();
        //let window = std::mem::replace(&mut self.window, None).unwrap();
        let on_resize = std::mem::replace(&mut self.on_resize_func, None).unwrap();
        let on_update = std::mem::replace(&mut self.on_update_fn, None).unwrap();
        if let Some(window) = self.window.as_ref() {
            let id = window.id().clone();
            event_loop.run(move |event, _, control_flow| {
                match event {
                    Event::WindowEvent {
                        ref event,
                        window_id,
                    } if window_id == id => match event {
                        WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
                        WindowEvent::Resized(physical_size) => {
                            // Handle resize
                            on_resize(WindowSize {
                                width: physical_size.width,
                                height: physical_size.height,
                            });
                        }
                        WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                            // Handle scale factor change
                            on_resize(WindowSize {
                                width: new_inner_size.width,
                                height: new_inner_size.height,
                            });
                        }
                        WindowEvent::KeyboardInput { input, .. } => unsafe {
                            if let Some(inp) = input::INPUT.as_mut() {
                                inp.handle_keyboard_event(input, id);
                            }
                        },
                        _ => {}
                    },
                    Event::MainEventsCleared => {
                        // update
                        on_update();
                        // update input
                        unsafe {
                            if let Some(input) = input::INPUT.as_mut() {
                                input.update_input();
                            }
                        }
                    }
                    _ => {}
                }
            });
        }
    }
}
pub fn get_new_window(settings: WindowInitSettings) -> WinitWindow {
    let winit_window = WinitWindow::new(settings);
    winit_window
}
