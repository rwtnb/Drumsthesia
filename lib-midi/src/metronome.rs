use std::time::Duration;

use midly::MidiMessage;

use crate::{MidiTrack, TempoTrack, MidiEvent, pulses_to_duration};

pub fn add_metronome(
    mut merged_track: MidiTrack,
    tempo_events: &TempoTrack,
    pulses_per_quarter_note: u16,
) -> MidiTrack {
    let track_id = 99;
    let beats_per_measure = 4;

    let mut events = Vec::new();
    let mut pulses = 0u64;

    let track_start = merged_track.events.first().unwrap().timestamp;
    let track_end = merged_track.events.last().unwrap().timestamp;
    let mut timestamp = track_start;

    for i in 0..i32::MAX {
        if i % beats_per_measure == 0 {
            events.push(MidiEvent {
                channel: 9,
                delta: 0,
                timestamp,
                message: MidiMessage::NoteOn {
                    key: 76.into(),
                    vel: 127.into(),
                },
                track_id,
            });
            events.push(MidiEvent {
                channel: 9,
                delta: 0,
                timestamp,
                message: MidiMessage::NoteOff {
                    key: 76.into(),
                    vel: 0.into(),
                },
                track_id,
            });
        } else {
            events.push(MidiEvent {
                channel: 9,
                delta: 0,
                timestamp,
                message: MidiMessage::NoteOn {
                    key: 77.into(),
                    vel: 80.into(),
                },
                track_id,
            });
            events.push(MidiEvent {
                channel: 9,
                delta: 0,
                timestamp,
                message: MidiMessage::NoteOff {
                    key: 76.into(),
                    vel: 0.into(),
                },
                track_id,
            });
        }

        pulses += pulses_per_quarter_note as u64 / 2;
        timestamp = pulses_to_duration(tempo_events, pulses, pulses_per_quarter_note);
        if timestamp >= track_end {
            break;
        }
    }

    let metronome = MidiTrack {
        notes: Vec::with_capacity(0),
        events,
        track_id,
    };

    for n in metronome.notes.iter().cloned() {
        merged_track.notes.push(n);
    }
    for e in metronome.events.iter().cloned() {
        merged_track.events.push(e);
    }

    merged_track
}
