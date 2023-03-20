use crate::{target::Target, OutputManager};
use midly::MidiMessage;
use std::{cell::RefCell, collections::HashSet, rc::Rc, time::Duration};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyboardInput, MouseButton},
};

mod rewind_controler;
use rewind_controler::RewindController;

use super::midi_mapping::get_midi_mapping_for_note;

pub struct MidiPlayer {
    playback: lib_midi::PlaybackState,
    rewind_controller: RewindController,
    pub output_manager: Rc<RefCell<OutputManager>>,
    midi_file: Rc<lib_midi::Midi>,
    wait_for_notes: WaitForNotes,
    guide_notes: bool,
    drums_volume: u8,
    music_volume: u8,
}

impl MidiPlayer {
    pub fn new(target: &mut Target) -> Self {
        let midi_file = target.midi_file.as_ref().unwrap();

        let mut player = Self {
            playback: lib_midi::PlaybackState::new(Duration::from_secs(3), &midi_file.merged_track),
            rewind_controller: RewindController::None,
            output_manager: target.output_manager.clone(),
            midi_file: midi_file.clone(),
            wait_for_notes: WaitForNotes::default(),
            guide_notes: target.config.guide_notes,
            drums_volume: target.config.drums_volume,
            music_volume: target.config.music_volume,
        };

        player.update(target, Duration::ZERO);

        player
    }

    /// When playing: returns midi events
    ///
    /// When paused: returns None
    pub fn update(
        &mut self,
        target: &mut Target,
        delta: Duration,
    ) -> Option<Vec<lib_midi::MidiEvent>> {
        rewind_controler::update(self, target);

        if delta.is_zero() {
            for channel in 0..16 {
                let value = if channel == 9 {
                    self.drums_volume
                } else {
                    self.music_volume
                };
                let msg = MidiMessage::Controller {
                    controller: midly::num::u7::new(7),
                    value: midly::num::u7::new(value),
                };
                target.output_manager.borrow_mut().midi_event(channel, msg)
            }
        }

        let elapsed = (delta / 10) * (target.config.speed_multiplier * 10.0) as u32;

        let events = self.playback.update(&self.midi_file.merged_track, elapsed);

        events.iter().for_each(|event| {
            let is_drum_channel = event.channel == midly::num::u7::new(9);
            let channel = event.channel;

            match event.message {
                MidiMessage::ProgramChange { program: _ } => {
                    self.output_manager
                        .borrow_mut()
                        .midi_event(channel, event.message);
                }
                MidiMessage::PitchBend { bend: _ } => {
                    self.output_manager
                        .borrow_mut()
                        .midi_event(channel, event.message);
                }
                MidiMessage::Controller {
                    controller: _,
                    value: _,
                } => {
                    self.output_manager
                        .borrow_mut()
                        .midi_event(channel, event.message);
                }
                MidiMessage::NoteOn { key, vel: _ } => {
                    if is_drum_channel {
                        self.wait_for_notes
                            .press_key(KeyPressSource::File, key.as_int(), true);
                    }

                    if !self.guide_notes && is_drum_channel {
                        return;
                    }

                    self.output_manager
                        .borrow_mut()
                        .midi_event(channel, event.message);
                }
                MidiMessage::NoteOff { key, .. } => {
                    if is_drum_channel {
                        self.wait_for_notes
                            .press_key(KeyPressSource::File, key.as_int(), false);
                    }

                    if !self.guide_notes && is_drum_channel {
                        return;
                    }

                    self.output_manager
                        .borrow_mut()
                        .midi_event(channel, event.message);
                }
                _ => {}
            }
        });

        if self.playback.is_paused() {
            None
        } else {
            Some(events)
        }
    }

    fn clear(&mut self) {
        let mut output = self.output_manager.borrow_mut();
        for note in self.playback.active_notes().iter() {
            output.midi_event(
                note.channel,
                MidiMessage::NoteOff {
                    key: note.key,
                    vel: midly::num::u7::new(0),
                },
            )
        }
    }
}

impl Drop for MidiPlayer {
    fn drop(&mut self) {
        self.clear();
    }
}

impl MidiPlayer {
    pub fn start(&mut self) {
        self.resume();
    }

    pub fn pause_resume(&mut self) {
        if self.playback.is_paused() {
            self.resume();
        } else {
            self.pause();
        }
    }

    pub fn pause(&mut self) {
        self.clear();
        self.playback.pause();
    }

    pub fn resume(&mut self) {
        self.playback.resume();
    }

    fn set_time(&mut self, time: Duration) {
        self.playback.set_time(time);

        let events = self
            .playback
            .update(&self.midi_file.merged_track, Duration::ZERO);
        std::mem::drop(events);

        self.clear();
    }

    pub fn rewind(&mut self, delta: i64) {
        let mut time = self.playback.time();

        if delta < 0 {
            let delta = Duration::from_millis((-delta) as u64);
            time = time.saturating_sub(delta);
        } else {
            let delta = Duration::from_millis(delta as u64);
            time = time.saturating_add(delta);
        }

        self.set_time(time);
    }

    pub fn set_percentage_time(&mut self, p: f32) {
        self.set_time(Duration::from_secs_f32(
            (p * self.playback.lenght().as_secs_f32()).max(0.0),
        ));
    }

    pub fn percentage(&self) -> f32 {
        self.playback.percentage()
    }

    pub fn time_without_lead_in(&self) -> f32 {
        self.playback.time().as_secs_f32() - self.playback.leed_in().as_secs_f32()
    }

    pub fn is_paused(&self) -> bool {
        self.playback.is_paused()
    }
}

impl MidiPlayer {
    pub fn keyboard_input(&mut self, input: &KeyboardInput) {
        rewind_controler::handle_keyboard_input(self, input)
    }

    pub fn mouse_input(&mut self, target: &mut Target, state: &ElementState, button: &MouseButton) {
        rewind_controler::handle_mouse_input(self, target, state, button)
    }

    pub fn handle_cursor_moved(&mut self, target: &mut Target, position: &PhysicalPosition<f64>) {
        rewind_controler::handle_cursor_moved(self, target, position)
    }
}

impl MidiPlayer {
    pub fn wait_for_notes(&mut self) -> &mut WaitForNotes {
        &mut self.wait_for_notes
    }
}

pub enum KeyPressSource {
    File,
    User,
}

#[derive(Default)]
pub struct WaitForNotes {
    required_notes: HashSet<u8>,
    played_notes: HashSet<u8>,
}

impl WaitForNotes {
    fn user_press_key(&mut self, note: u8, active: bool) {
        if let Some(mapping) = get_midi_mapping_for_note(note) {
            if active {
                self.played_notes.insert(mapping.id);
            }
        }
    }

    fn file_press_key(&mut self, note: u8, active: bool) {
        if let Some(mapping) = get_midi_mapping_for_note(note) {
            if active {
                self.required_notes.insert(mapping.id);
            }
        }
    }

    fn check_and_drain(&mut self) {
        if !self.required_notes.is_empty() && self.required_notes.is_subset(&self.played_notes) {
            self.required_notes.drain();
            self.played_notes.drain();
        }
    }

    pub fn press_key(&mut self, src: KeyPressSource, note: u8, active: bool) {
        match src {
            KeyPressSource::User => self.user_press_key(note, active),
            KeyPressSource::File => self.file_press_key(note, active),
        }
    }

    pub fn are_required_keys_pressed(&mut self) -> bool {
        self.check_and_drain();
        self.required_notes.is_empty()
    }
}
