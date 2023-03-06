use neothesia_pipelines::quad::{QuadInstance, QuadPipeline};
use std::{
    borrow::{Borrow, BorrowMut},
    time::Duration,
};
use wgpu_jumpstart::Color;
use winit::{
    dpi::LogicalSize,
    event::{KeyboardInput, WindowEvent},
};

use self::{
    marks::Marks,
    midi_mapping::{get_midi_mapping_for_note, MidiMapping},
};

use super::{Scene, SceneType};
use crate::{midi_event::MidiEvent, target::Target, NeothesiaEvent};

mod drum_roll;
mod marks;
use drum_roll::DrumRoll;

mod notes;
use notes::Notes;

mod midi_player;
use midi_player::MidiPlayer;

mod midi_mapping;

mod toast_manager;
use toast_manager::ToastManager;

pub struct PlayingScene {
    drum_roll: DrumRoll,
    notes: Notes,
    marks: Marks,
    player: MidiPlayer,
    played_notes: Vec<(f32, MidiMapping)>,
    quad_pipeline: QuadPipeline,
    toast_manager: ToastManager,
}

impl PlayingScene {
    pub fn new(target: &mut Target) -> Self {
        let track_notes = target
            .midi_file
            .as_ref()
            .unwrap()
            .merged_track
            .notes
            .iter()
            .filter(|i| i.channel == 9)
            .map(|i| i.note)
            .collect();

        let drum_roll = DrumRoll::new(
            track_notes,
            &target.gpu,
            &target.transform_uniform,
            target.window_state.logical_size,
        );

        let player = MidiPlayer::new(target);

        let mut marks = Marks::new(target, drum_roll.lanes());
        marks.update(target, player.time_without_lead_in());

        let mut notes = Notes::new(target, drum_roll.lanes());
        notes.update(target, player.time_without_lead_in());

        Self {
            drum_roll,
            notes,
            marks,
            player,
            played_notes: Default::default(),
            quad_pipeline: QuadPipeline::new(&target.gpu, &target.transform_uniform),

            toast_manager: ToastManager::default(),
        }
    }

    fn update_progresbar(&mut self, target: &mut Target) {
        let ww = target.window_state.logical_size.width;
        let size_x = ww * self.player.percentage();
        self.quad_pipeline.update_instance_buffer(
            &target.gpu.queue,
            vec![
                QuadInstance {
                    position: [0.0, 0.0],
                    size: [ww, 5.0],
                    color: Color::from_rgba8(100, 100, 100, 1.0).into_linear_rgba(),
                    ..Default::default()
                },
                QuadInstance {
                    position: [0.0, 0.0],
                    size: [size_x, 5.0],
                    color: Color::from_rgba8(200, 200, 200, 1.0).into_linear_rgba(),
                    ..Default::default()
                },
            ],
        );
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
        self.notes.resize(target, self.drum_roll.lanes());
        self.marks
            .resize(target, self.drum_roll.lanes(), &self.played_notes);
    }

    fn update(&mut self, target: &mut Target, delta: Duration) {
        let play_along = self.player.play_along();
        if play_along.are_required_keys_pressed() || !target.config.play_along {
            if self.player.update(target, delta).is_none() {
                self.drum_roll.reset_notes();
            }
        }

        if self.player.percentage() >= 1.0 {
            self.player.pause();
        }

        self.update_progresbar(target);

        self.notes.update(
            target,
            self.player.time_without_lead_in() + target.config.playback_offset,
        );

        self.marks.update(
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
            .render(&target.transform_uniform, &mut render_pass);

        self.marks
            .render(&target.transform_uniform, &mut render_pass);

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

    fn midi_event(&mut self, target: &mut Target, event: &MidiEvent) {
        match event {
            MidiEvent::NoteOn { key, .. } => {
                self.player.play_along().press_key(
                    midi_player::KeyPressSource::User,
                    *key,
                    true,
                );

                if let Some(mapping) = get_midi_mapping_for_note(*key) {
                    self.played_notes.push((
                        self.player.time_without_lead_in() + target.config.playback_offset,
                        mapping,
                    ));
                    self.marks.resize(target, self.drum_roll.lanes(), &self.played_notes);
                }
            }
            MidiEvent::NoteOff { key, .. } => self.player.play_along().press_key(
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
