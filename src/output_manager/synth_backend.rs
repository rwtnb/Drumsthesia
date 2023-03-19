use std::{error::Error, path::Path, sync::mpsc::Receiver, time::Duration};

use crate::output_manager::{OutputConnection, OutputDescriptor};

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use lib_midi::MidiEvent;
use midly::MidiMessage;

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
                gain: 0.1,
                ..Default::default()
            })
            .unwrap();

            let mut file = std::fs::File::open(path).unwrap();
            let font = oxisynth::SoundFont::load(&mut file).unwrap();
            synth.add_font(font, true);
            synth.set_sample_rate(sample_rate);
            synth.program_reset();

            move || {
                let (l, r) = synth.read_next();

                if let Ok(evt) = rx.try_recv() {
                    let channel = evt.channel;
                    match evt.message {
                        MidiMessage::ProgramChange { program } => {
                            synth
                                .send_event(oxisynth::MidiEvent::ProgramChange {
                                    channel,
                                    program_id: program.as_int(),
                                })
                                .ok();
                        }
                        MidiMessage::NoteOn { key, vel } => {
                            synth
                                .send_event(oxisynth::MidiEvent::NoteOn {
                                    channel,
                                    key: key.as_int(),
                                    vel: vel.as_int(),
                                })
                                .ok();
                        }
                        MidiMessage::NoteOff { key, vel: _ } => {
                            synth
                                .send_event(oxisynth::MidiEvent::NoteOff {
                                    channel,
                                    key: key.as_int(),
                                })
                                .ok();
                        }
                        _ => {
                            log::warn!("implement missing midi messages {:?}", evt.message)
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
    fn midi_event(&mut self, channel: u8, message: MidiMessage) {
        let event = MidiEvent {
            channel,
            message,
            delta: 0,
            timestamp: Duration::ZERO,
            track_id: 0
        };
        self.tx.send(event).ok();
    }
}
