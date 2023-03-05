use std::{error::Error, path::Path, sync::mpsc::Receiver};

use crate::output_manager::{OutputConnection, OutputDescriptor};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use iced_native::program;

#[cfg(all(feature = "fluid-synth", not(feature = "oxi-synth")))]
const SAMPLES_SIZE: usize = 1410;

enum MidiEvent {
    NoteOn { ch: u8, key: u8, vel: u8 },
    NoteOff { ch: u8, key: u8 },
    ProgramChange { ch: u8, program: u8 },
}

pub struct SynthBackend {
    _host: cpal::Host,
    device: cpal::Device,

    stream_config: cpal::StreamConfig,
    sample_format: cpal::SampleFormat,
}

impl SynthBackend {
    pub fn new() -> Result<Self, Box<dyn Error>> {
        let host = cpal::default_host();

        let device = host
            .default_output_device()
            .ok_or("failed to find a default output device")?;

        let config = device.default_output_config()?;
        let sample_format = config.sample_format();

        let stream_config: cpal::StreamConfig = config.into();

        Ok(Self {
            _host: host,
            device,

            stream_config,
            sample_format,
        })
    }

    fn run<T: cpal::Sample>(&self, rx: Receiver<MidiEvent>, path: &Path) -> cpal::Stream {
        let mut next_value = {
            let sample_rate = self.stream_config.sample_rate.0 as f32;

            let mut synth = oxisynth::Synth::new(oxisynth::SynthDescriptor {
                sample_rate,
                gain: 0.3,
                ..Default::default()
            })
            .unwrap();

            let font_id = {
                let mut file = std::fs::File::open(path).unwrap();
                let font = oxisynth::SoundFont::load(&mut file).unwrap();
                synth.add_font(font, true)
            };

            synth.program_reset();

            move || {
                let (l, r) = synth.read_next();

                if let Ok(e) = rx.try_recv() {
                    match e {
                        MidiEvent::ProgramChange { ch, program } => {
                            match synth.program_select(ch, font_id, 0, program) {
                                Ok(()) => {},
                                Err(err) => log::error!("{}", err)
                            }
                        }
                        MidiEvent::NoteOn { ch, key, vel } => {
                            synth
                                .send_event(oxisynth::MidiEvent::NoteOn {
                                    channel: ch as _,
                                    key: key as _,
                                    vel: vel as _,
                                })
                                .ok();
                        }
                        MidiEvent::NoteOff { ch, key } => {
                            synth
                                .send_event(oxisynth::MidiEvent::NoteOff {
                                    channel: ch as _,
                                    key: key as _,
                                })
                                .ok();
                        }
                    }
                }

                (l, r)
            }
        };

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let channels = self.stream_config.channels as usize;

        let stream = self
            .device
            .build_output_stream(
                &self.stream_config,
                move |output: &mut [T], _: &cpal::OutputCallbackInfo| {
                    for frame in output.chunks_mut(channels) {
                        let (l, r) = next_value();

                        let l: T = cpal::Sample::from::<f32>(&l);
                        let r: T = cpal::Sample::from::<f32>(&r);

                        let channels = [l, r];

                        for (id, sample) in frame.iter_mut().enumerate() {
                            *sample = channels[id % 2];
                        }
                    }
                },
                err_fn,
            )
            .unwrap();
        stream.play().unwrap();

        stream
    }

    pub fn new_output_connection(&mut self, path: &Path) -> SynthOutputConnection {
        let (tx, rx) = std::sync::mpsc::channel::<MidiEvent>();
        let _stream = match self.sample_format {
            cpal::SampleFormat::F32 => self.run::<f32>(rx, path),
            cpal::SampleFormat::I16 => self.run::<i16>(rx, path),
            cpal::SampleFormat::U16 => self.run::<u16>(rx, path),
        };

        SynthOutputConnection { _stream, tx }
    }

    pub fn get_outputs(&self) -> Vec<OutputDescriptor> {
        vec![OutputDescriptor::Synth(None)]
    }
}

pub struct SynthOutputConnection {
    _stream: cpal::Stream,
    tx: std::sync::mpsc::Sender<MidiEvent>,
}

impl OutputConnection for SynthOutputConnection {
    fn midi_event(&mut self, msg: midi::Message) {
        match msg {
            midi::ProgramChange(ch, program) => {
                self.tx
                    .send(MidiEvent::ProgramChange {
                        ch: ch as u8,
                        program,
                    })
                    .ok();
            }
            midi::NoteOn(ch, key, vel) => {
                self.tx
                    .send(MidiEvent::NoteOn {
                        ch: ch as u8,
                        key,
                        vel,
                    })
                    .ok();
            }
            midi::NoteOff(ch, key, _vel) => {
                self.tx.send(MidiEvent::NoteOff { ch: ch as u8, key }).ok();
            }
            _ => {}
        }
    }
}
