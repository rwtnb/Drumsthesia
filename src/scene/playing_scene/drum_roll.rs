use std::collections::HashMap;

use crate::{
    config::Config,
    utils::{Point, Size},
    TransformUniform, Uniform,
};

use neothesia_pipelines::quad::{QuadInstance, QuadPipeline};
use piano_math::range::KeyboardRange;
use wgpu_glyph::{GlyphBrush, Section};

mod lane;
pub use lane::Lane;
use wgpu_jumpstart::Gpu;

pub struct DrumRoll {
    pos: Point<f32>,
    size: Size<f32>,

    lanes: Vec<Lane>,
    range: KeyboardRange,

    quad_pipeline: QuadPipeline,
    should_reupload: bool,
}

impl DrumRoll {
    pub fn new(
        gpu: &Gpu,
        transform_uniform: &Uniform<TransformUniform>,
        window_size: winit::dpi::LogicalSize<f32>,
    ) -> Self {
        let range = KeyboardRange::standard_88_keys();

        let quad_pipeline = QuadPipeline::new(gpu, transform_uniform);
        let lanes: Vec<Lane> = range.iter().map(|_| Lane::new()).collect();

        let mut drum_roll = Self {
            pos: Point { x: 0.0, y: 5.0 },
            size: Default::default(),

            lanes,
            range,

            quad_pipeline,
            should_reupload: false,
        };

        drum_roll.resize(window_size);
        drum_roll
    }

    pub fn keys(&self) -> &[Lane] {
        &self.lanes
    }

    /// Calculate positions of keys
    fn calculate_positions(&mut self) {
        let neutral_height = self.size.h / self.range.count() as f32;
        let keyboard = piano_math::standard_88_keys(self.size.w, neutral_height);

        for (id, key) in keyboard.keys.iter().enumerate() {
            self.lanes[id].note_id = key.note_id();

            self.lanes[id].pos = self.pos;
            self.lanes[id].pos.y += key.y();

            self.lanes[id].size = key.size().into();
        }

        self.queue_reupload();
    }

    pub fn resize(&mut self, window_size: winit::dpi::LogicalSize<f32>) {
        self.size.w = window_size.width * 0.2;
        self.size.h = window_size.height - 5.0;

        self.pos.x = 0.0;
        self.pos.y = 5.0;

        self.calculate_positions();
    }

    pub fn user_midi_event(&mut self, event: &crate::MidiEvent) {
        match event {
            crate::MidiEvent::NoteOn { key, .. } => {
                if self.range.contains(*key) {
                    let id = *key as usize - 27;
                    let key = &mut self.lanes[id];

                    key.set_pressed_by_user(true);
                    self.queue_reupload();
                }
            }
            crate::MidiEvent::NoteOff { key, .. } => {
                if self.range.contains(*key) {
                    let id = *key as usize - 27;
                    let key = &mut self.lanes[id];

                    key.set_pressed_by_user(false);
                    self.queue_reupload();
                }
            }
        }
    }

    pub fn file_midi_events(&mut self, config: &Config, events: &[lib_midi::MidiEvent]) {
        for e in events {
            match e.message {
                lib_midi::midly::MidiMessage::NoteOn { key, .. } => {
                    let key = key.as_int();

                    if self.range.contains(key) && e.channel == 9 {
                        let id = key as usize - 27;
                        let key = &mut self.lanes[id];

                        let color = &config.color_schema[e.track_id % config.color_schema.len()];
                        self.queue_reupload();
                    }
                }
                lib_midi::midly::MidiMessage::NoteOff { key, .. } => {
                    let key = key.as_int();
                    if self.range.contains(key) && e.channel == 9 {
                        let id = key as usize - 27;
                        let key = &mut self.lanes[id];

                        self.queue_reupload();
                    }
                }
                _ => continue,
            };
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

            // black_background
            instances.push(QuadInstance {
                position: self.pos.into(),
                size: self.size.into(),
                color: [0.2, 0.2, 0.2, 1.0],
                ..Default::default()
            });

            for key in self.lanes.iter() {
                instances.push(QuadInstance::from(key));
            }
        });
        self.should_reupload = false;
    }

    pub fn update(&mut self, queue: &wgpu::Queue, brush: &mut GlyphBrush<()>) {
        if self.should_reupload {
            self.reupload(queue);
        }

        for (id, key) in self.lanes.iter().enumerate() {
            let Point { x, y } = key.pos;
            let Size { w, h } = key.size;

            let size = h * 0.7;

            brush.queue(Section {
                screen_position: (x + 5.0, y + (size / 4.0)),
                text: vec![wgpu_glyph::Text::new(key.label())
                    .with_color([1.0, 1.0, 1.0, 0.5])
                    .with_scale(size)],
                bounds: (f32::INFINITY, h),
                layout: wgpu_glyph::Layout::default()
                    .h_align(wgpu_glyph::HorizontalAlign::Left)
                    .v_align(wgpu_glyph::VerticalAlign::Top),
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
