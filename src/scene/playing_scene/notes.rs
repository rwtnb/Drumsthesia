use std::time::Duration;

use crate::target::Target;
use crate::TransformUniform;
use crate::Uniform;
use neothesia_pipelines::waterfall::{NoteInstance, WaterfallPipeline};
use wgpu_jumpstart::Color;

use super::drum_roll::Lane;
use super::midi_mapping::get_midi_mappings;
use super::midi_mapping::midi_mappings_count;

pub struct Notes {
    notes_pipeline: WaterfallPipeline,
}

impl Notes {
    pub fn new(target: &mut Target, lanes: &Vec<Lane>) -> Self {
        let notes_pipeline = WaterfallPipeline::new(
            &target.gpu,
            &target.transform_uniform,
            target.midi_file.as_ref().unwrap().merged_track.notes.len(),
        );
        let mut notes = Self { notes_pipeline };
        notes.resize(target, lanes);
        notes
    }

    pub fn resize(&mut self, target: &mut Target, lanes: &Vec<Lane>) {
        let midi = &target.midi_file.as_ref().unwrap();
        let mut instances = Vec::new();

        for note in midi.merged_track.notes.iter() {
            if note.channel != 9 {
                continue;
            }

            match lanes.iter().find(|i| i.mapping.accept_note(note.note)) {
                None => {
                    println!("missing mapping for note {}", note.note);
                }

                Some(lane) => {
                    let note_duration = note.duration.as_secs_f32() * 4.0;
                    let color: Color = lane.mapping.color.into();
                    let note_height = lane.size.h * 0.6;
                    let note_width = note_duration * note_height;
                    let note_width = if note_width <= note_height * 0.6 {
                        note_height * 0.5
                    } else if note_width <= note_height * 0.8 {
                        note_height * 0.75
                    } else {
                        note_height
                    };

                    let x = note.start.as_secs_f32();
                    let y = lane.pos.y + (lane.size.h / 2.0) - (note_height / 2.0);

                    instances.push(NoteInstance {
                        position: [x, y],
                        size: [note_width, note_height],
                        color: color.into_linear_rgb(),
                        radius: note_height * 0.1,
                    });
                }
            }
        }

        self.notes_pipeline
            .update_instance_buffer(&mut target.gpu, instances);
    }

    pub fn update(&mut self, target: &mut Target, time: f32) {
        self.notes_pipeline.update_time(&mut target.gpu, time);
    }

    pub fn render<'rpass>(
        &'rpass mut self,
        transform_uniform: &'rpass Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        self.notes_pipeline.render(transform_uniform, render_pass);
    }
}
