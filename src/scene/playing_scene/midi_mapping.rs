use crate::config::{default_color_schema, ColorSchema};

#[derive(Clone, Copy, Hash)]
pub struct MidiMapping {
    pub id: u8,
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

const MAPPINGS: [MidiMapping; 12] = [
    MidiMapping {
        id: 0,
        name: "Crash Cymbal 1",
        note: 49,
        alt_note: 55,
        color: COLOR_SCHEMA.orange1,
    },
    MidiMapping {
        id: 1,
        name: "Ride Cymbal",
        note: 51,
        alt_note: 59,
        color: COLOR_SCHEMA.orange2,
    },
    MidiMapping {
        id: 2,
        name: "Crash Cymbal 2",
        note: 57,
        alt_note: 52,
        color: COLOR_SCHEMA.orange3,
    },
    MidiMapping {
        id: 3,
        name: "Ride Bell",
        note: 53,
        alt_note: 53,
        color: COLOR_SCHEMA.orange4,
    },
    MidiMapping {
        id: 4,
        name: "High Tom",
        note: 48,
        alt_note: 50,
        color: COLOR_SCHEMA.purple1,
    },
    MidiMapping {
        id: 5,
        name: "Mid Tom",
        note: 47,
        alt_note: 45,
        color: COLOR_SCHEMA.purple2,
    },
    MidiMapping {
        id: 6,
        name: "Low Tom",
        note: 41,
        alt_note: 43,
        color: COLOR_SCHEMA.purple3,
    },
    MidiMapping {
        id: 7,
        name: "Open Hi-Hat",
        note: 46,
        alt_note: 26,
        color: COLOR_SCHEMA.green,
    },
    MidiMapping {
        id: 8,
        name: "Closed Hi-Hat",
        note: 42,
        alt_note: 22,
        color: COLOR_SCHEMA.beige,
    },
    MidiMapping {
        id: 9,
        name: "Snare Drum",
        note: 38,
        alt_note: 40,
        color: COLOR_SCHEMA.blue,
    },
    MidiMapping {
        id: 10,
        name: "Pedal Hi-Hat",
        note: 44,
        alt_note: 44,
        color: COLOR_SCHEMA.cyan,
    },
    MidiMapping {
        id: 11,
        name: "Bass Drum",
        note: 35,
        alt_note: 36,
        color: COLOR_SCHEMA.yellow,
    },
];

pub fn get_all_midi_mappings() -> Vec<MidiMapping> {
    vec![
        MAPPINGS[0],
        MAPPINGS[10],
        MAPPINGS[7],
        MAPPINGS[8],
        MAPPINGS[9],
        MAPPINGS[4],
        MAPPINGS[5],
        MAPPINGS[11],
        MAPPINGS[6],
        MAPPINGS[1],
        MAPPINGS[3],
        MAPPINGS[2],
    ]
}

pub fn get_midi_mappings(notes: Vec<u8>) -> Vec<MidiMapping> {
    MAPPINGS.iter()
        .filter(|m| m.accept_notes(notes.clone()))
        .map(|m| *m)
        .collect()
}

pub const fn get_midi_mapping(id: u8) -> MidiMapping {
    MAPPINGS[id as usize]
}

pub fn get_midi_mapping_for_note(note: u8) -> Option<MidiMapping> {
    MAPPINGS.iter()
        .find(|i| i.accept_note(note))
        .map(|i| *i)
}

pub const fn midi_mappings_count() -> usize {
    MAPPINGS.len()
}
