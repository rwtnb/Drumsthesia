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
    pub fn new(target: &mut Target, lanes: &[Lane]) -> Self {
        let notes_pipeline = WaterfallPipeline::new(
            &target.gpu,
            &target.transform_uniform,
            target.midi_file.as_ref().unwrap().merged_track.notes.len(),
        );
        let mut notes = Self { notes_pipeline };
        notes.resize(target, lanes);
        notes
    }

    pub fn resize(&mut self, target: &mut Target, lanes: &[Lane]) {
        let midi = &target.midi_file.as_ref().unwrap();

        let mut instances = Vec::new();

        for note in midi.merged_track.notes.iter() {
            let lane = lanes.iter().find(|i| i.notes.contains(&note.note));

            if lane.is_some() && note.channel == 9 {
                let lane = lane.unwrap();
                let color_schema = &target.config.color_schema;

                let color = &color_schema[note.track_id % color_schema.len()];
                let color = color.base;
                let color: Color = color.into();

                instances.push(NoteInstance {
                    position: [note.start.as_secs_f32(), lane.y_position() + 4.0],
                    size: [0.1, lane.height() - 10.0],
                    color: color.into_linear_rgb(),
                    radius: 0.0,
                });
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
