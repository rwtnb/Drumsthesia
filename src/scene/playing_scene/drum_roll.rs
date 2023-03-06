use std::collections::HashMap;

use crate::{
    utils::{Point, Size},
    TransformUniform, Uniform,
};

use neothesia_pipelines::quad::{QuadInstance, QuadPipeline};
use wgpu_glyph::{GlyphBrush, Section};

mod lane;
pub use lane::Lane;
use wgpu_jumpstart::Gpu;

use super::midi_mapping::{get_midi_mapping_for_note, get_midi_mappings};

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
        self.size.h = window_size.height - 5.0;

        self.pos.x = 0.0;
        self.pos.y = 5.0;

        self.calculate_positions();
    }

    pub fn user_midi_event(&mut self, event: &crate::MidiEvent) {
        match event {
            crate::MidiEvent::NoteOn { key, .. } => {
                if let Some(lane) = self.lanes.iter_mut().find(|i| i.mapping.id == *key) {
                    lane.set_pressed_by_user(true);
                    self.queue_reupload();
                }
            }
            crate::MidiEvent::NoteOff { key, .. } => {
                if let Some(lane) = self.lanes.iter_mut().find(|i| i.mapping.id == *key) {
                    lane.set_pressed_by_user(false);
                    self.queue_reupload();
                }
            }
        }
    }

    pub fn reset_notes(&mut self) {
        self.queue_reupload();
    }

    fn queue_reupload(&mut self) {
        self.should_reupload = true;
    }

    /// Reupload instances to GPU
    fn reupload(&mut self, queue: &wgpu::Queue) {
        self.quad_pipeline.with_instances_mut(queue, |instances| {
            instances.clear();

            instances.push(QuadInstance {
                position: [self.size.w * 0.33333, 5.0],
                size: [1.0, self.size.h],
                color: [0.88, 0.67, 0.03, 0.5],
                ..Default::default()
            });

            for (i, lane) in self.lanes.iter().enumerate() {
                let lane_color = if i % 2 == 0 {
                    [1.0, 1.0, 1.0, 0.02]
                } else {
                    [1.0, 1.0, 1.0, 0.022]
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
                    color: [1.0, 1.0, 1.0, 0.024],
                    ..Default::default()
                });

                instances.push(QuadInstance::from(lane));
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
