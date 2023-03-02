use neothesia_pipelines::quad::{QuadInstance, QuadPipeline};
use std::time::Duration;
use wgpu_jumpstart::Color;
use winit::{
    dpi::LogicalSize,
    event::{KeyboardInput, WindowEvent},
};

use super::{Scene, SceneType};
use crate::{midi_event::MidiEvent, target::Target, NeothesiaEvent};

mod drum_roll;
use drum_roll::DrumRoll;

mod notes;
use notes::Notes;

mod midi_player;
use midi_player::MidiPlayer;

mod toast_manager;
use toast_manager::ToastManager;

pub struct PlayingScene {
    drum_roll: DrumRoll,
    notes: Notes,
    player: MidiPlayer,
    quad_pipeline: QuadPipeline,
    toast_manager: ToastManager,
}

impl PlayingScene {
    pub fn new(target: &mut Target) -> Self {
        let drum_roll = DrumRoll::new(
            &target.gpu,
            &target.transform_uniform,
            target.window_state.logical_size,
        );

        let mut notes = Notes::new(target, drum_roll.keys());

        let player = MidiPlayer::new(target);
        notes.update(target, player.time_without_lead_in());

        Self {
            drum_roll,
            notes,
            player,
            quad_pipeline: QuadPipeline::new(&target.gpu, &target.transform_uniform),

            toast_manager: ToastManager::default(),
        }
    }

    fn update_progresbar(&mut self, target: &mut Target) {
        let size_x = target.window_state.logical_size.width * self.player.percentage();
        self.quad_pipeline.update_instance_buffer(
            &target.gpu.queue,
            vec![QuadInstance {
                position: [0.0, 0.0],
                size: [size_x, 5.0],
                color: Color::from_rgba8(56, 145, 255, 1.0).into_linear_rgba(),
                ..Default::default()
            }],
        );
    }

    #[cfg(feature = "record")]
    pub fn playback_progress(&self) -> f32 {
        self.player.percentage() * 100.0
    }
}

impl Scene for PlayingScene {
    fn scene_type(&self) -> SceneType {
        SceneType::Playing
    }

    fn start(&mut self) {
        self.player.start();
    }

    fn resize(&mut self, target: &mut Target) {
        let (width, height) = target.window_state.logical_size.into();
        self.drum_roll.resize(LogicalSize::new(width, height - 5.0));
        self.notes.resize(target, self.drum_roll.keys());
    }

    fn update(&mut self, target: &mut Target, delta: Duration) {
        if self.player.play_along().are_required_keys_pressed() || !target.config.play_along {
            if let Some(midi_events) = self.player.update(target, delta) {
                self.drum_roll
                    .file_midi_events(&target.config, &midi_events);
            } else {
                self.drum_roll.reset_notes();
            }
        }

        self.update_progresbar(target);

        self.notes.update(
            target,
            self.player.time_without_lead_in() + target.config.playback_offset,
        );

        self.drum_roll
            .update(&target.gpu.queue, target.text_renderer.glyph_brush());
        self.toast_manager.update(target);
    }

    fn render(&mut self, target: &mut Target, view: &wgpu::TextureView) {
        let mut render_pass = target
            .gpu
            .encoder
            .begin_render_pass(&wgpu::RenderPassDescriptor {
                label: None,
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Load,
                        store: true,
                    },
                })],
                depth_stencil_attachment: None,
            });

        self.notes
            .render(&target.transform_uniform, &mut render_pass);

        self.drum_roll
            .render(&target.transform_uniform, &mut render_pass);

        self.quad_pipeline
            .render(&target.transform_uniform, &mut render_pass)
    }

    fn window_event(&mut self, target: &mut Target, event: &WindowEvent) {
        use winit::event::WindowEvent::*;
        use winit::event::{ElementState, VirtualKeyCode};

        match &event {
            KeyboardInput { input, .. } => {
                self.player.keyboard_input(input);

                settings_keyboard_input(target, &mut self.toast_manager, input);

                if input.state == ElementState::Released {
                    match input.virtual_keycode {
                        Some(VirtualKeyCode::Escape) => {
                            target.proxy.send_event(NeothesiaEvent::GoBack);
                        }
                        Some(VirtualKeyCode::Space) => {
                            self.player.pause_resume();
                        }
                        _ => {}
                    }
                }
            }
            MouseInput { state, button, .. } => {
                self.player.mouse_input(target, state, button);
            }
            CursorMoved { position, .. } => {
                self.player.handle_cursor_moved(target, position);
            }
            _ => {}
        }
    }

    fn midi_event(&mut self, _target: &mut Target, event: &MidiEvent) {
        match event {
            MidiEvent::NoteOn { key, .. } => self.player.play_along_mut().press_key(
                midi_player::KeyPressSource::User,
                *key,
                true,
            ),
            MidiEvent::NoteOff { key, .. } => self.player.play_along_mut().press_key(
                midi_player::KeyPressSource::User,
                *key,
                false,
            ),
        }

        self.drum_roll.user_midi_event(event);
    }
}

fn settings_keyboard_input(
    target: &mut Target,
    toast_manager: &mut ToastManager,
    input: &KeyboardInput,
) {
    use winit::event::{ElementState, VirtualKeyCode};

    if input.state != ElementState::Released {
        return;
    }

    let virtual_keycode = if let Some(virtual_keycode) = input.virtual_keycode {
        virtual_keycode
    } else {
        return;
    };

    match virtual_keycode {
        VirtualKeyCode::Up | VirtualKeyCode::Down => {
            let amount = if target.window_state.modifers_state.shift() {
                0.5
            } else {
                0.1
            };

            if virtual_keycode == VirtualKeyCode::Up {
                target.config.speed_multiplier += amount;
            } else {
                target.config.speed_multiplier -= amount;
                target.config.speed_multiplier = target.config.speed_multiplier.max(0.0);
            }

            toast_manager.speed_toast(target.config.speed_multiplier);
        }

        VirtualKeyCode::Minus | VirtualKeyCode::Plus | VirtualKeyCode::Equals => {
            let amount = if target.window_state.modifers_state.shift() {
                0.1
            } else {
                0.01
            };

            if virtual_keycode == VirtualKeyCode::Minus {
                target.config.playback_offset -= amount;
            } else {
                target.config.playback_offset += amount;
            }

            toast_manager.offset_toast(target.config.playback_offset);
        }

        _ => {}
    }
}
