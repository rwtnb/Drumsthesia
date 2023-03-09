use crate::config::default_color_schema;
use crate::target::Target;
use neothesia_pipelines::waterfall::{NoteInstance, WaterfallPipeline};
use wgpu_jumpstart::{Color, TransformUniform, Uniform};

use super::drum_roll::Lane;
use super::midi_mapping::MidiMapping;

pub struct Marks {
    pipeline: WaterfallPipeline,
    played_notes: Vec<(f32, MidiMapping)>,
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
            played_notes: Default::default(),
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
        let window_height = target.window_state.logical_size.height;
        let mut instances = Vec::new();

        for (time, mapping) in played_notes {
            if let Some(lane) = lanes.iter().find(|i| i.mapping.id == mapping.id) {
                let color: Color = default_color_schema().red.into();
                let x = *time;
                let h = lane.height() * 0.1;
                let w = h;
                let y = lane.y_position() + (lane.height() / 2.0) - (h / 2.0);

                instances.push(NoteInstance {
                    position: [x, y],
                    size: [w, h],
                    color: color.into_linear_rgb(),
                    radius: h,
                    spacing: (window_height / 720.0) * 2.0,
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
