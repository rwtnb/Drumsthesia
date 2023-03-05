use crate::config::{default_color_schema, ColorSchema};

#[derive(Clone, Copy, Hash)]
pub struct MidiMapping {
    pub name: &'static str,
    pub note: u8,
    pub alt_note: u8,
    pub color: (u8, u8, u8),
}

impl MidiMapping {
    pub fn accept_notes(&self, notes: Vec<u8>) -> bool {
        notes.contains(&self.note) || notes.contains(&self.alt_note)
    }

    pub fn accept_note(&self, note: u8) -> bool {
        self.note == note || self.alt_note == note
    }
}

impl PartialEq for MidiMapping {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name 
            && self.note == other.note 
            && self.alt_note == other.alt_note
    }
}

impl Eq for MidiMapping { }

const COLOR_SCHEMA: ColorSchema = default_color_schema();

const MAPPINGS: [MidiMapping; 11] = [
    MidiMapping {
        name: "Crash Cymbal 1",
        note: 49,
        alt_note: 55,
        color: COLOR_SCHEMA.orange1,
    },
    MidiMapping {
        name: "Ride Cymbal",
        note: 51,
        alt_note: 59,
        color: COLOR_SCHEMA.orange2,
    },
    MidiMapping {
        name: "Crash Cymbal 2",
        note: 57,
        alt_note: 52,
        color: COLOR_SCHEMA.orange3,
    },
    MidiMapping {
        name: "High Tom",
        note: 48,
        alt_note: 50,
        color: COLOR_SCHEMA.purple1,
    },
    MidiMapping {
        name: "Mid Tom",
        note: 47,
        alt_note: 45,
        color: COLOR_SCHEMA.purple2,
    },
    MidiMapping {
        name: "Low Tom",
        note: 41,
        alt_note: 43,
        color: COLOR_SCHEMA.purple3,
    },
    MidiMapping {
        name: "Open Hi-Hat",
        note: 46,
        alt_note: 26,
        color: COLOR_SCHEMA.green,
    },
    MidiMapping {
        name: "Closed Hi-Hat",
        note: 42,
        alt_note: 22,
        color: COLOR_SCHEMA.beige,
    },
    MidiMapping {
        name: "Snare Drum",
        note: 38,
        alt_note: 40,
        color: COLOR_SCHEMA.blue,
    },
    MidiMapping {
        name: "Pedal Hi-Hat",
        note: 44,
        alt_note: 44,
        color: COLOR_SCHEMA.cyan,
    },
    MidiMapping {
        name: "Bass Drum",
        note: 35,
        alt_note: 36,
        color: COLOR_SCHEMA.yellow,
    },
];

pub const fn get_midi_mapping(id: usize) -> MidiMapping {
    MAPPINGS[id]
}

pub fn get_midi_mapping_for_note(note: u8) -> Option<usize> {
    MAPPINGS.iter()
        .enumerate()
        .find(|i| i.1.accept_note(note))
        .map(|i| i.0)
}
