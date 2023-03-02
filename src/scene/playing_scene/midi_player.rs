use crate::{target::Target, OutputManager};
use num::FromPrimitive;
use std::{cell::RefCell, collections::HashSet, rc::Rc, time::Duration};
use winit::{
    dpi::PhysicalPosition,
    event::{ElementState, KeyboardInput, MouseButton},
};

use crate::midi_event::MidiEvent;

mod rewind_controler;
use rewind_controler::RewindController;

pub struct MidiPlayer {
    playback: lib_midi::PlaybackState,
    rewind_controller: RewindController,
    output_manager: Rc<RefCell<OutputManager>>,
    midi_file: Rc<lib_midi::Midi>,
    play_along: PlayAlong,
}

impl MidiPlayer {
    pub fn new(target: &mut Target) -> Self {
        let midi_file = target.midi_file.as_ref().unwrap();

        let mut player = Self {
            playback: lib_midi::PlaybackState::new(Duration::from_secs(3), &midi_file.merged_track),
            rewind_controller: RewindController::None,
            output_manager: target.output_manager.clone(),
            midi_file: midi_file.clone(),
            play_along: PlayAlong::default(),
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

        let elapsed = (delta / 10) * (target.config.speed_multiplier * 10.0) as u32;

        let events = self.playback.update(&self.midi_file.merged_track, elapsed);

        events.iter().for_each(|event| {
            use lib_midi::midly::MidiMessage;
            let channel = event.channel;

            match event.message {
                MidiMessage::NoteOn { key, vel } => {
                    let event = midi::Message::NoteOn(
                        midi::Channel::from_u8(event.channel).unwrap(),
                        key.as_int(),
                        vel.as_int(),
                    );
                    self.output_manager.borrow_mut().midi_event(event);
                    if channel == 9 {
                        self.play_along
                            .press_key(KeyPressSource::File, key.as_int(), true);
                    }
                }
                MidiMessage::NoteOff { key, .. } => {
                    let event = midi::Message::NoteOff(
                        midi::Channel::from_u8(event.channel).unwrap(),
                        key.as_int(),
                        0,
                    );
                    self.output_manager.borrow_mut().midi_event(event);
                    if channel == 9 {
                        self.play_along
                            .press_key(KeyPressSource::File, key.as_int(), false);
                    }
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
                MidiEvent::NoteOff {
                    channel: note.channel,
                    key: note.key,
                }
                .into(),
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

        // Discard all of the events till that point
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
    pub fn play_along(&self) -> &PlayAlong {
        &self.play_along
    }

    pub fn play_along_mut(&mut self) -> &mut PlayAlong {
        &mut self.play_along
    }
}

pub enum KeyPressSource {
    File,
    User,
}

#[derive(Debug, Default)]
pub struct PlayAlong {
    required_notes: HashSet<u8>,
}

impl PlayAlong {
    fn user_press_key(&mut self, note_id: u8, active: bool) {
        if active {
            self.required_notes.remove(&note_id);
        }
    }

    fn file_press_key(&mut self, note_id: u8, active: bool) {
        if active {
            self.required_notes.insert(note_id);
        } else {
            self.required_notes.remove(&note_id);
        }
    }

    pub fn press_key(&mut self, src: KeyPressSource, note_id: u8, active: bool) {
        match src {
            KeyPressSource::User => self.user_press_key(note_id, active),
            KeyPressSource::File => self.file_press_key(note_id, active),
        }
    }

    pub fn are_required_keys_pressed(&self) -> bool {
        self.required_notes.is_empty()
    }
}

// #[cfg(feature = "play_along")]
// use std::sync::{mpsc, Arc, Mutex};

// #[cfg(feature = "play_along")]
// struct PlayAlongController {
//     _midi_in_conn: midir::MidiInputConnection<()>,
//     midi_in_rec: mpsc::Receiver<(bool, u8, u8)>,

//     input_pressed_keys: [bool; 88],
//     required_notes: Arc<Mutex<HashMap<u8, MidiNote>>>,
//     waiting_for_note: bool,
// }

// #[cfg(feature = "play_along")]
// impl PlayAlongController {
//     fn new() -> Option<Self> {
//         let input_pressed_keys = [false; 88];
//         let required_notes = Arc::new(Mutex::new(HashMap::new()));

//         let (tx, midi_in_rec) = mpsc::channel();

//         let _midi_in_conn = {
//             let midi_in = midir::MidiInput::new("Neothesia-in").unwrap();
//             let in_ports = midi_in.ports();

//             use std::io::{stdin, stdout, Write};

//             let in_port = match in_ports.len() {
//                 0 => return None,
//                 1 => {
//                     println!(
//                         "Choosing the only available input port: {}",
//                         midi_in.port_name(&in_ports[0]).unwrap()
//                     );
//                     &in_ports[0]
//                 }
//                 _ => {
//                     println!("\nAvailable input ports:");
//                     for (i, p) in in_ports.iter().enumerate() {
//                         println!("{}: {}", i, midi_in.port_name(p).unwrap());
//                     }
//                     print!("Please select input port: ");
//                     stdout().flush().unwrap();
//                     let mut input = String::new();
//                     stdin().read_line(&mut input).unwrap();
//                     in_ports
//                         .get(input.trim().parse::<usize>().unwrap())
//                         .ok_or("invalid input port selected")
//                         .unwrap()
//                 }
//             };

//             let required_notes = required_notes.clone();

//             midi_in
//                 .connect(
//                     in_port,
//                     "neothesia-read-input",
//                     move |_, message, _| {
//                         if message.len() == 3 {
//                             let note = message[1];
//                             if note >= 21 && note <= 108 {
//                                 if message[0] == 128 || message[2] == 0 {
//                                     tx.send((false, message[1], message[2])).unwrap();
//                                 } else if message[0] == 144 {
//                                     required_notes.lock().unwrap().remove(&note);
//                                     tx.send((true, message[1], message[2])).unwrap();
//                                 }
//                             }
//                         }
//                     },
//                     (),
//                 )
//                 .unwrap()
//         };

//         Some(Self {
//             _midi_in_conn,
//             midi_in_rec,

//             input_pressed_keys,
//             required_notes,
//             waiting_for_note: false,
//         })
//     }

//     fn update(&mut self, target: &mut Target, notes_state: &mut Vec<MidiEvent>, timer: &mut Timer) {
//         if let Ok(event) = self.midi_in_rec.try_recv() {
//             if event.0 {
//                 self.input_pressed_keys[event.1 as usize - 21] = true;
//                 target
//                     .output_manager
//                     .borrow_mut()
//                     .midi_event(midi::Message::NoteOn(midi::Ch1, event.1, event.2));

//                 notes_state.push(MidiEvent::NoteOn {
//                     channel: 0,
//                     track_id: 0,
//                     key: event.1,
//                     vel: event.2,
//                 });
//             } else {
//                 self.input_pressed_keys[event.1 as usize - 21] = false;
//                 target
//                     .output_manager
//                     .borrow_mut()
//                     .midi_event(midi::Message::NoteOff(midi::Ch1, event.1, event.2));

//                 notes_state.push(MidiEvent::NoteOff {
//                     channel: 0,
//                     key: event.1,
//                 });
//             }
//         }
//         if self.required_notes.lock().unwrap().len() == 0 && self.waiting_for_note == true {
//             self.waiting_for_note = false;
//             timer.resume();
//         }
//     }

//     fn require_note(&mut self, timer: &mut Timer, n: &MidiNote) {
//         if n.note >= 21 && n.note <= 108 && n.ch != 9 {
//             self.required_notes
//                 .lock()
//                 .unwrap()
//                 .insert(n.note, n.clone());
//             self.waiting_for_note = true;
//             timer.pause();
//         }
//     }

//     fn clear(&mut self) {
//         self.required_notes.lock().unwrap().clear();
//     }
// }
