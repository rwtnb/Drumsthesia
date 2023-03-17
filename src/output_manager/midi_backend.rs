use std::collections::HashSet;

use crate::output_manager::{OutputConnection, OutputDescriptor};


use lib_midi::ActiveNote;
use midly::{MidiMessage, live::LiveEvent, num::u7};

pub struct MidiOutputConnection {
    conn: midi_io::MidiOutputConnection,
    active_notes: HashSet<ActiveNote>,
}

impl From<midi_io::MidiOutputConnection> for MidiOutputConnection {
    fn from(conn: midi_io::MidiOutputConnection) -> Self {
        Self {
            conn,
            active_notes: Default::default(),
        }
    }
}

pub struct MidiBackend {
    manager: midi_io::MidiOutputManager,
}

impl MidiBackend {
    pub fn new() -> Result<Self, midi_io::InitError> {
        Ok(Self {
            manager: midi_io::MidiOutputManager::new()?,
        })
    }

    pub fn get_outputs(&self) -> Vec<OutputDescriptor> {
        let mut outs = Vec::new();
        for (id, port) in self.manager.outputs().into_iter().enumerate() {
            outs.push(OutputDescriptor::MidiOut(MidiPortInfo { id, port }))
        }
        outs
    }

    pub fn new_output_connection(port: &MidiPortInfo) -> Option<MidiOutputConnection> {
        midi_io::MidiOutputManager::connect_output(port.port.clone())
            .map(MidiOutputConnection::from)
    }
}

impl OutputConnection for MidiOutputConnection {
    fn midi_event(&mut self, channel: u8, message: MidiMessage) {
        match &message {
            MidiMessage::ProgramChange { program: _ } => {
                let event = LiveEvent::Midi {
                    channel: channel.into(),
                    message
                };
                let mut data = Vec::new();
                event.write(&mut data).unwrap();
                self.conn.send(&data).ok();
            },
            MidiMessage::NoteOn { key, vel: _ } => {
                let event = LiveEvent::Midi {
                    channel: channel.into(),
                    message
                };
                let mut data = Vec::new();
                event.write(&mut data).unwrap();
                self.conn.send(&data).ok();
                self.active_notes.insert(ActiveNote { key: *key, channel });
            },
            MidiMessage::NoteOff { key, vel: _ } => {
                let event = LiveEvent::Midi {
                    channel: channel.into(),
                    message
                };
                let mut data = Vec::new();
                event.write(&mut data).unwrap();
                self.conn.send(&data).ok();
                self.active_notes.remove(&ActiveNote { key: *key, channel });
            }
            _ => {}
        }
    }
}

impl Drop for MidiOutputConnection {
    fn drop(&mut self) {
        for note in self.active_notes.iter() {
                let key = u7::try_from(note.key.into()).unwrap();
                let event = LiveEvent::Midi {
                    channel: note.channel.into(),
                    message: MidiMessage::NoteOff { key, vel: u7::new(0) }
                };
                let mut data = Vec::new();
                event.write(&mut data).unwrap();
                self.conn.send(&data).ok();
        }
    }
}

#[derive(Clone, Debug, Eq)]
pub struct MidiPortInfo {
    id: usize,
    port: midi_io::MidiOutputPort,
}

impl PartialEq for MidiPortInfo {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id && self.port == other.port
    }
}

impl std::fmt::Display for MidiPortInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.port)
    }
}
