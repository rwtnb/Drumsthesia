use std::time::Duration;

use lib_midi::MidiEvent;
use midly::{live::LiveEvent, MidiMessage};

use crate::{EventLoopProxy, NeothesiaEvent};

pub struct InputManager {
    input: midi_io::MidiInputManager,
    tx: EventLoopProxy,
    current_connection: Option<midi_io::MidiInputConnection>,
}

impl InputManager {
    pub fn new(tx: EventLoopProxy) -> Self {
        let input = midi_io::MidiInputManager::new().unwrap();
        Self {
            input,
            tx,
            current_connection: None,
        }
    }

    pub fn inputs(&self) -> Vec<midi_io::MidiInputPort> {
        self.input.inputs()
    }

    pub fn connect_input(&mut self, port: midi_io::MidiInputPort) {
        let tx = self.tx.clone();
        self.current_connection = midi_io::MidiInputManager::connect_input(port, move |message| {
            let event = LiveEvent::parse(message).unwrap();
            match &event {
                LiveEvent::Midi { channel: _, message } => match message {
                    MidiMessage::NoteOn { key: _, vel: _ } => {
                        let event = MidiEvent {
                            channel: 9,
                            message: *message,
                            delta: 0,
                            timestamp: Duration::ZERO,
                            track_id: 0,
                        };
                        tx.proxy.send_event(NeothesiaEvent::MidiInput(event)).unwrap();
                    }
                    MidiMessage::NoteOff { key: _, vel: _ } => {
                        let event = MidiEvent {
                            channel: 9,
                            message: *message,
                            delta: 0,
                            timestamp: Duration::ZERO,
                            track_id: 0,
                        };
                        tx.proxy.send_event(NeothesiaEvent::MidiInput(event)).unwrap();
                    }
                    _ => {}
                },
                _ => {}
            }
        });
    }
}
