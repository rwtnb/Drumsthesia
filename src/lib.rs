pub mod ui;

pub mod scene;

pub mod utils;

pub mod output_manager;

pub mod input_manager;

pub mod config;

pub mod target;

use std::{
    borrow::{Borrow, BorrowMut},
    rc::Rc,
};

use futures::Future;
use instant::Duration;
use js_sys::Uint8Array;
use lib_midi::MidiEvent;
use midly::{live::LiveEvent, MidiMessage};
use scene::{playing_scene, scene_manager, SceneType};
use target::Target;
use utils::window::WindowState;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Blob, Request, RequestInit, RequestMode, Response};
use wgpu_jumpstart::{Gpu, Surface};
use winit::{
    event::WindowEvent,
    event_loop::{ControlFlow, EventLoop, EventLoopBuilder},
    window::Window,
};

#[derive(Debug, Clone)]
pub struct EventLoopProxy {
    #[cfg(feature = "app")]
    proxy: winit::event_loop::EventLoopProxy<NeothesiaEvent>,
}

impl EventLoopProxy {
    #[cfg(feature = "app")]
    pub fn new_winit(proxy: winit::event_loop::EventLoopProxy<NeothesiaEvent>) -> Self {
        Self { proxy }
    }

    #[track_caller]
    pub fn send_event(&self, event: NeothesiaEvent) {
        #[cfg(feature = "app")]
        self.proxy.send_event(event).unwrap();
    }
}

#[derive(Debug, Clone, Copy)]
pub enum NeothesiaEvent {
    #[cfg(feature = "app")]
    MainMenu,
    MidiInput(MidiEvent),
    GoBack,
}

pub fn block_on<F>(f: F) -> <F as Future>::Output
where
    F: Future,
{
    futures::executor::block_on(f)
}

pub struct Drumsthesia {
    pub target: Target,
    surface: Surface,

    last_time: instant::Instant,
    pub game_scene: scene_manager::SceneManager,
}

impl Drumsthesia {
    pub fn new(mut target: Target, surface: Surface) -> Self {
        let to = playing_scene::PlayingScene::new(&mut target);
        let mut game_scene = scene_manager::SceneManager::new(to);

        target.resize();
        game_scene.resize(&mut target);
        target.gpu.submit();

        Self {
            target,
            surface,
            last_time: instant::Instant::now(),
            game_scene,
        }
    }

    pub fn window_event(&mut self, event: &WindowEvent, control_flow: &mut ControlFlow) {
        self.target.window_state.window_event(event);

        match &event {
            WindowEvent::Resized(_) => {
                self.surface.resize_swap_chain(
                    &self.target.gpu.device,
                    self.target.window_state.physical_size.width,
                    self.target.window_state.physical_size.height,
                );

                self.target.resize();
                self.game_scene.resize(&mut self.target);

                self.target.gpu.submit();
            }
            WindowEvent::ScaleFactorChanged { .. } => {
                // TODO: Check if this update is needed;
                self.target.resize();
                self.game_scene.resize(&mut self.target);
            }
            WindowEvent::CloseRequested => *control_flow = ControlFlow::Exit,
            _ => {}
        }

        self.game_scene.window_event(&mut self.target, event);
    }

    pub fn midi_event(&mut self, event: &MidiEvent) {
        self.game_scene.midi_event(&mut self.target, event);
    }

    pub fn neothesia_event(&mut self, event: &NeothesiaEvent, control_flow: &mut ControlFlow) {
        match event {
            NeothesiaEvent::MainMenu => {
                    let input = midir::MidiInput::new("MidiIo-in").unwrap();

                    let ports = input.ports();
                    log::info!("ports: {}", ports.len());
                    let port = ports.get(0).unwrap();

                    let _conn = input
                        .connect(
                            &port,
                            "MidiIo-in-conn",
                            move |_, message, _| {
                                let event = LiveEvent::parse(message).unwrap();
                                log::info!("{:?}", event);
                                match &event {
                                    LiveEvent::Midi {
                                        channel: _,
                                        message,
                                    } => match message {
                                        MidiMessage::NoteOn { key, vel } => {
                                            let event = MidiEvent {
                                                channel: 9,
                                                message: *message,
                                                delta: 0,
                                                timestamp: Duration::ZERO,
                                                track_id: 0,
                                            };
                                            log::info!("evt {} {}", key, vel);
                                        }
                                        MidiMessage::NoteOff { key, vel } => {
                                            let event = MidiEvent {
                                                channel: 9,
                                                message: *message,
                                                delta: 0,
                                                timestamp: Duration::ZERO,
                                                track_id: 0,
                                            };
                                            log::info!("evt {} {}", key, vel);
                                        }
                                        _ => {}
                                    },
                                    _ => {}
                                }
                            },
                            (),
                        )
                        .unwrap();
            }
            NeothesiaEvent::GoBack => match self.game_scene.scene_type() {
                SceneType::MainMenu => {
                    *control_flow = ControlFlow::Exit;
                }
                SceneType::Playing => {
                }
            },
            NeothesiaEvent::MidiInput(event) => self.midi_event(event),
        }
    }

    pub fn update(&mut self) {
        let delta = self.last_time.elapsed();
        self.last_time = instant::Instant::now();

        self.game_scene.update(&mut self.target, delta);
    }

    pub fn render(&mut self) {
        let frame = loop {
            let swap_chain_output = self.surface.get_current_texture();
            match swap_chain_output {
                Ok(s) => break s,
                Err(err) => log::warn!("{:?}", err),
            }
        };

        let view = &frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        self.target
            .gpu
            .clear(view, self.target.config.background_color.into());

        self.game_scene.render(&mut self.target, view);
        self.target.text_renderer.render(
            (
                self.target.window_state.logical_size.width,
                self.target.window_state.logical_size.height,
            ),
            &mut self.target.gpu,
            view,
        );

        self.target.gpu.submit();
        frame.present();
    }
}

async fn run(event_loop: EventLoop<NeothesiaEvent>, window: Window) {
    let window_state = WindowState::new(&window);
    let instance = wgpu::Instance::default();

    let size = window.inner_size();
    let (gpu, surface) = Gpu::for_window(&instance, &window, size.width, size.height)
        .await
        .unwrap();

    let proxy = EventLoopProxy::new_winit(event_loop.create_proxy());
    let tx = proxy.clone();

    let midi_file = lib_midi::Midi::new(include_bytes!("./../test.mid")).unwrap();
    let mut target = Target::new(Some(Rc::new(midi_file)), window_state, proxy, gpu);
    {
        let om = target.output_manager.borrow_mut();
        let mut om = om.as_ref().borrow_mut();
        let out = om.outputs();
        //om.connect(output_manager::OutputDescriptor::Synth(None));
    }

    let mut app = Drumsthesia::new(target, surface);
    //tx.proxy.send_event(NeothesiaEvent::MainMenu);

    event_loop.run(move |event, _, control_flow| {
        use winit::event::Event;
        match &event {
            Event::UserEvent(event) => {
                app.neothesia_event(event, control_flow);
            }
            Event::MainEventsCleared => {
                app.game_scene.main_events_cleared(&mut app.target);

                app.update();
                window.request_redraw();
            }
            Event::WindowEvent { event, .. } => {
                app.window_event(event, control_flow);
            }
            Event::RedrawRequested(_) => {
                app.render();
            }
            _ => {}
        }
    });
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen(start))]
fn main() {
    let builder = winit::window::WindowBuilder::new().with_inner_size(winit::dpi::LogicalSize {
        width: 1080.0,
        height: 720.0,
    });

    let event_loop = EventLoopBuilder::with_user_event().build();

    let builder = builder.with_title("Drumsthesia");

    let window = builder.build(&event_loop).unwrap();

    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init().expect("could not initialize logger");
    use winit::platform::web::WindowExtWebSys;
    // On wasm, append the canvas to the document body
    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| doc.body())
        .and_then(|body| {
            body.append_child(&web_sys::Element::from(window.canvas()))
                .ok()
        })
        .expect("couldn't append canvas to document body");
    wasm_bindgen_futures::spawn_local(run(event_loop, window));
}
