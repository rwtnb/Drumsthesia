use crate::{
    utils::{Point, Size},
    TransformUniform, Uniform,
};

use neothesia_pipelines::quad::{QuadInstance, QuadPipeline};
use wgpu_glyph::{GlyphBrush, Section};

use wgpu_jumpstart::Gpu;

use super::midi_mapping::{get_midi_mappings, MidiMapping};

pub struct DrumRoll {
    pos: Point<f32>,
    size: Size<f32>,
    lanes: Vec<Lane>,
    quad_pipeline: QuadPipeline,
    should_reupload: bool,
}

impl DrumRoll {
    pub fn new(
        track_notes: Vec<u8>,
        gpu: &Gpu,
        transform_uniform: &Uniform<TransformUniform>,
        window_size: winit::dpi::LogicalSize<f32>,
    ) -> Self {
        let quad_pipeline = QuadPipeline::new(gpu, transform_uniform);
        let lanes = get_midi_mappings(track_notes.clone())
            .iter()
            .map(|m| Lane::new(*m))
            .collect();

        let mut drum_roll = Self {
            pos: Point { x: 0.0, y: 5.0 },
            size: Default::default(),
            lanes,
            quad_pipeline,
            should_reupload: false,
        };

        drum_roll.resize(window_size);
        drum_roll
    }

    pub fn lanes(&self) -> &Vec<Lane> {
        &self.lanes
    }

    /// Calculate positions of keys
    fn calculate_positions(&mut self) {
        let lane_height = self.size.h / self.lanes.len() as f32;
        let mut offset = 0.0;

        for lane in self.lanes.iter_mut() {
            lane.pos = self.pos;
            lane.pos.y += offset;

            lane.size = Size {
                w: self.size.w * 0.33333,
                h: lane_height,
            };

            offset += lane_height;
        }

        self.queue_reupload();
    }

    pub fn resize(&mut self, window_size: winit::dpi::LogicalSize<f32>) {
        self.size.w = window_size.width;
        self.size.h = window_size.height;

        self.pos.x = 0.0;
        self.pos.y = 5.0;

        self.calculate_positions();
    }

    fn queue_reupload(&mut self) {
        self.should_reupload = true;
    }

    /// Reupload instances to GPU
    fn reupload(&mut self, queue: &wgpu::Queue) {
        self.quad_pipeline.with_instances_mut(queue, |instances| {
            instances.clear();

            for (i, lane) in self.lanes.iter().enumerate() {
                let lane_color = if i % 2 == 0 {
                    [0.02, 0.02, 0.02, 1.0]
                } else {
                    [0.022, 0.022, 0.022, 1.0]
                };

                instances.push(QuadInstance {
                    position: lane.pos.into(),
                    size: [self.size.w, lane.size.h],
                    color: lane_color,
                    ..Default::default()
                });

                instances.push(QuadInstance {
                    position: [lane.pos.x, lane.pos.y - 1.0],
                    size: [self.size.w, 1.0],
                    color: [0.05, 0.05, 0.05, 1.0],
                    ..Default::default()
                });
            }
        });
        self.should_reupload = false;
    }

    pub fn update(&mut self, queue: &wgpu::Queue, brush: &mut GlyphBrush<()>) {
        if self.should_reupload {
            self.reupload(queue);
        }

        for lane in &self.lanes {
            let Point { x, y } = lane.pos;
            let Size { w, h } = lane.size;

            let size = h * 0.1;

            brush.queue(Section {
                screen_position: (x + w - size, y + h - size),
                text: vec![wgpu_glyph::Text::new(lane.label().to_uppercase().as_str())
                    .with_color([1.0, 1.0, 1.0, 0.2])
                    .with_scale(size)],
                bounds: (w, h),
                layout: wgpu_glyph::Layout::default()
                    .h_align(wgpu_glyph::HorizontalAlign::Right)
                    .v_align(wgpu_glyph::VerticalAlign::Bottom),
            })
        }
    }

    pub fn render<'rpass>(
        &'rpass mut self,
        transform_uniform: &'rpass Uniform<TransformUniform>,
        render_pass: &mut wgpu::RenderPass<'rpass>,
    ) {
        self.quad_pipeline.render(transform_uniform, render_pass);
    }
}

pub struct Lane {
    pub pos: Point<f32>,
    pub size: Size<f32>,
    pub mapping: MidiMapping,
}

impl Lane {
    pub fn new(mapping: MidiMapping) -> Self {
        Self {
            pos: Default::default(),
            size: Default::default(),
            mapping,
        }
    }

    pub fn label(&self) -> &str {
        self.mapping.name
    }
}
