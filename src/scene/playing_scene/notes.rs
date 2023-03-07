use crate::target::Target;
use crate::TransformUniform;
use crate::Uniform;
use neothesia_pipelines::waterfall::{NoteInstance, WaterfallPipeline};
use wgpu_jumpstart::Color;

use super::drum_roll::Lane;

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
        let window_height = target.window_state.logical_size.height;
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
                    let color: Color = lane.mapping.color.into();
                    let x = note.start.as_secs_f32();
                    let h = lane.height() * 0.5;
                    let y = lane.y_position() + (lane.height() / 2.0) - (h / 2.0);
                    let w = if note.velocity <= 40 {
                        lane.height() * 0.25
                    } else {
                        h
                    };

                    instances.push(NoteInstance {
                        position: [x, y],
                        size: [w, h],
                        color: color.into_linear_rgb(),
                        radius: h * 0.1,
                        spacing: (window_height / 720.0) * 2.0,
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
