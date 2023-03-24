use midly::{MidiMessage, num::u7};

use crate::{pulses_to_duration, MidiEvent, MidiTrack, TempoTrack};

pub fn add_metronome(
    mut merged_track: MidiTrack,
    tempo_events: &TempoTrack,
    pulses_per_quarter_note: u16,
) -> MidiTrack {
    let beats_per_measure = 4;

    let mut events = Vec::new();
    let mut pulses = 0u64;

    let track_start = merged_track.events.first().unwrap().timestamp;
    let track_end = merged_track.events.last().unwrap().timestamp;
    let mut timestamp = track_start;

    let track_id = 99;
    let channel = 15;
    let delta = 0;
    let accent_key: u7 =  64.into();
    let regular_key: u7 = 62.into();
    let wood_block: u7 = 115.into();

    events.push(MidiEvent {
        channel,
        delta,
        timestamp,
        message: MidiMessage::ProgramChange { program: wood_block },
        track_id,
    });

    for i in 0..i32::MAX {
        if i % beats_per_measure == 0 {
            events.push(MidiEvent {
                channel,
                delta,
                timestamp,
                message: MidiMessage::NoteOn {
                    key: accent_key,
                    vel: 127.into(),
                },
                track_id,
            });
            events.push(MidiEvent {
                channel,
                delta,
                timestamp,
                message: MidiMessage::NoteOff {
                    key: accent_key,
                    vel: 0.into(),
                },
                track_id,
            });
        } else {
            events.push(MidiEvent {
                channel,
                delta,
                timestamp,
                message: MidiMessage::NoteOn {
                    key: regular_key,
                    vel: 80.into(),
                },
                track_id,
            });
            events.push(MidiEvent {
                channel,
                delta,
                timestamp,
                message: MidiMessage::NoteOff {
                    key: regular_key,
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
