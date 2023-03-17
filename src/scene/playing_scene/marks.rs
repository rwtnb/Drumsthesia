use crate::config::default_color_schema;
use crate::target::Target;
use crate::TransformUniform;
use crate::Uniform;
use neothesia_pipelines::waterfall::{NoteInstance, WaterfallPipeline};
use wgpu_jumpstart::Color;

use super::drum_roll::Lane;
use super::midi_mapping::MidiMapping;
use super::midi_mapping::midi_mappings_count;

pub struct Marks {
    pipeline: WaterfallPipeline,
}

impl Marks {
    pub fn new(target: &mut Target, lanes: &Vec<Lane>) -> Self {
        let pipeline = WaterfallPipeline::new(
            &target.gpu,
            &target.transform_uniform,
            target.midi_file.as_ref().unwrap().merged_track.notes.len(),
        );
        let mut marks = Self {
            pipeline,
        };
        marks.resize(target, lanes, &Default::default());
        marks
    }

    pub fn resize(
        &mut self,
        target: &mut Target,
        lanes: &Vec<Lane>,
        played_notes: &Vec<(f32, MidiMapping)>,
    ) {
        let mut instances = Vec::new();

        for (time, mapping) in played_notes {
            if let Some(lane) = lanes.iter().find(|i| i.mapping.id == mapping.id) {
                let color: Color = default_color_schema().red.into();
                let x = *time;
                let h = lane.size.h * 0.1;
                let w = h;
                let y = lane.pos.y + (lane.size.h / 2.0) - (h / 2.0);

                instances.push(NoteInstance {
                    position: [x, y],
                    size: [w, h],
                    color: color.into_linear_rgb(),
                    radius: h,
                });
            }
        }

        self.pipeline
            .update_instance_buffer(&mut target.gpu, instances);
    }

    pub fn update(&mut self, target: &mut Target, time: f32) {
        self.pipeline.update_time(&mut target.gpu, time);
    }

    pub fn render<'rpass>(
        &'rpass mut self,
        transform_uniform: &'rpass Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        self.pipeline.render(transform_uniform, render_pass);
    }
}
