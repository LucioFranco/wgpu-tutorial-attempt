use std::time::{Duration, Instant};

use controller::CameraController;
use glam::{vec3, Quat};
use tokio::runtime::Runtime;
use winit::{
    dpi::PhysicalSize,
    event::{DeviceEvent, ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::{Window, WindowBuilder},
};

mod camera;
mod controller;
mod model;
mod render;
mod texture;

use render::Render;

fn main() -> anyhow::Result<()> {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new().build(&event_loop).unwrap();

    let mut game = Game::new(&window)?;
    let mut last_render_time = Instant::now();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        match event {
            Event::DeviceEvent { ref event, .. } => {
                game.input(event);
            }

            Event::WindowEvent {
                ref event,
                window_id,
            } if window_id == window.id() => match event {
                WindowEvent::CloseRequested
                | WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state: ElementState::Pressed,
                            virtual_keycode: Some(VirtualKeyCode::Escape),
                            ..
                        },
                    ..
                } => *control_flow = ControlFlow::Exit,
                WindowEvent::KeyboardInput {
                    input:
                        KeyboardInput {
                            state,
                            virtual_keycode: Some(key),
                            ..
                        },
                    ..
                } => {
                    game.controller.process_keyboard(*key, *state);
                }
                WindowEvent::Resized(physical_size) => {
                    game.resize(*physical_size);
                }
                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    game.resize(**new_inner_size);
                }
                _ => {}
            },

            Event::RedrawRequested(_) => {
                let now = Instant::now();
                let dt = now - last_render_time;
                last_render_time = now;

                game.update(dt);

                match game.render() {
                    Ok(_) => {}
                    Err(e) => match e.downcast::<wgpu::SurfaceError>() {
                        Ok(wgpu::SurfaceError::Lost) => game.resize(game.size),
                        Ok(wgpu::SurfaceError::OutOfMemory) => *control_flow = ControlFlow::Exit,
                        Ok(e) => eprintln!("Unexpected surface error: {:?}", e),
                        Err(e) => eprintln!("Unexpected Error: {:?}", e),
                    },
                }
            }
            Event::MainEventsCleared => {
                // RedrawRequested will only trigger once, unless we manually
                // request it.
                window.request_redraw();
            }
            _ => {}
        }
    });
}

struct Game {
    size: PhysicalSize<u32>,
    render: Render,
    controller: CameraController,
    mouse_pressed: bool,
    rt: Runtime,
}

impl Game {
    fn new(window: &Window) -> anyhow::Result<Self> {
        let rt = tokio::runtime::Builder::new_current_thread().build()?;

        let render = rt.block_on(Render::new(window));

        let controller = CameraController::new(4.0, 0.4);

        Ok(Self {
            size: window.inner_size(),
            render,
            rt,
            mouse_pressed: false,
            controller,
        })
    }

    fn resize(&mut self, new_size: PhysicalSize<u32>) {
        self.render.resize(new_size);
    }

    fn input(&mut self, event: &DeviceEvent) -> bool {
        match event {
            DeviceEvent::Key(KeyboardInput {
                virtual_keycode: Some(key),
                state,
                ..
            }) => self.controller.process_keyboard(*key, *state),

            DeviceEvent::MouseWheel { delta, .. } => {
                self.controller.process_scroll(delta);
                true
            }
            DeviceEvent::Button { button: 1, state } => {
                self.mouse_pressed = *state == ElementState::Pressed;
                true
            }
            DeviceEvent::MouseMotion { delta } => {
                if self.mouse_pressed {
                    self.controller.process_mouse(delta.0, delta.1);
                }
                true
            }
            _ => false,
        }
    }

    fn update(&mut self, dt: Duration) {
        let camera = self.render.camera_mut();
        self.controller.update_camera(camera, dt);

        let light_uniform = self.render.light_mut();
        let old_position = light_uniform.position;
        light_uniform.position =
            Quat::from_axis_angle(vec3(0.0, 1.0, 0.0), (60.0 * dt.as_secs_f32()).to_radians())
                * old_position;

        self.render.update();
    }

    fn render(&mut self) -> anyhow::Result<()> {
        self.render.render()
    }
}
