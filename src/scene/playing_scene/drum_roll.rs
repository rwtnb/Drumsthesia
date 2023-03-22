use crate::{
    utils::{Point, Size},
    TransformUniform, Uniform
};

use neothesia_pipelines::quad::{QuadInstance, QuadPipeline};
use wgpu_glyph::{GlyphBrush, Section};

use wgpu_jumpstart::Gpu;

use super::midi_mapping::{get_midi_mappings, MidiMapping, get_all_midi_mappings};

pub struct DrumRoll {
    pos: Point<f32>,
    size: Size<f32>,
    lanes: Vec<Lane>,
    quad_pipeline: QuadPipeline,
    should_reupload: bool,
    is_vertical_layout: bool,
}

impl DrumRoll {
    pub fn new(
        track_notes: Vec<u8>,
        gpu: &Gpu,
        transform_uniform: &Uniform<TransformUniform>,
        window_size: winit::dpi::LogicalSize<f32>,
        is_vertical_layout: bool,
    ) -> Self {
        let quad_pipeline = QuadPipeline::new(gpu, transform_uniform);

        let lanes = if is_vertical_layout {
            get_all_midi_mappings().iter().map(|m| Lane::new(*m)).collect()
        } else {
            get_midi_mappings(track_notes)
            .iter()
            .map(|m| Lane::new(*m))
            .collect()
        };

        let mut drum_roll = Self {
            pos: Point { x: 0.0, y: 5.0 },
            size: Size { w: window_size.width, h: window_size.height - 5.0 },
            lanes,
            quad_pipeline,
            should_reupload: false,
            is_vertical_layout,
        };

        drum_roll.resize(window_size);
        drum_roll
    }

    pub fn lanes(&self) -> &Vec<Lane> {
        &self.lanes
    }

    fn calculate_positions_horizontal(&mut self) {
        let lane_height = f32::min(self.size.h / self.lanes.len() as f32, 110.0);
        let mut offset = (self.size.h / 2.0) - (lane_height * self.lanes.len() as f32 / 2.0);

        for lane in self.lanes.iter_mut() {
            lane.pos = self.pos;
            lane.pos.y += offset;

            lane.size = Size {
                w: self.size.w,
                h: lane_height,
            };

            offset += lane_height;
        }

        self.queue_reupload();
    }

    fn calculate_positions_vertical(&mut self) {
        let lane_width = f32::min(self.size.w / self.lanes.len() as f32, 110.0);
        let mut offset = (self.size.w / 2.0) - (lane_width * self.lanes.len() as f32 / 2.0);

        for lane in self.lanes.iter_mut() {
            lane.pos = self.pos;
            lane.pos.x += offset;

            lane.size = Size {
                w: lane_width,
                h: self.size.h,
            };

            offset += lane_width;
        }

        self.queue_reupload();
    }

    pub fn resize(&mut self, window_size: winit::dpi::LogicalSize<f32>) {
        self.size.w = window_size.width;
        self.size.h = window_size.height - 5.0;

        self.pos.x = 0.0;
        self.pos.y = 5.0;

        if self.is_vertical_layout {
            self.calculate_positions_vertical();
        } else {
            self.calculate_positions_horizontal();
        }
    }

    fn queue_reupload(&mut self) {
        self.should_reupload = true;
    }

    fn reupload(&mut self, queue: &wgpu::Queue) {
        self.quad_pipeline.with_instances_mut(queue, |instances| {
            let lane_border_color = [0.05, 0.05, 0.05, 1.0];

            instances.clear();
            instances.push(QuadInstance {
                position: self.pos.into(),
                size: self.size.into(),
                color: [0.01, 0.01, 0.01, 1.0],
                ..Default::default()
            });

            for (i, lane) in self.lanes.iter().enumerate() {
                let lane_color = if i % 2 == 0 {
                    [0.02, 0.02, 0.02, 1.0]
                } else {
                    [0.022, 0.022, 0.022, 1.0]
                };

                instances.push(QuadInstance {
                    position: lane.pos.into(),
                    size: [lane.size.w, lane.size.h],
                    color: lane_color,
                    ..Default::default()
                });

                if self.is_vertical_layout {
                    instances.push(QuadInstance {
                        position: [lane.pos.x, lane.pos.y],
                        size: [1.0, self.size.h],
                        color: lane_border_color,
                        ..Default::default()
                    });
                } else {
                    instances.push(QuadInstance {
                        position: [lane.pos.x, lane.pos.y],
                        size: [self.size.w, 1.0],
                        color: lane_border_color,
                        ..Default::default()
                    });
                }
            }

            let lane = self.lanes.last().unwrap();
            if self.is_vertical_layout {
                instances.push(QuadInstance {
                    position: [lane.pos.x + lane.size.w, lane.pos.y],
                    size: [1.0, self.size.h],
                    color: lane_border_color,
                    ..Default::default()
                });
            } else {
                instances.push(QuadInstance {
                    position: [lane.pos.x, lane.pos.y + lane.size.h],
                    size: [self.size.w, 1.0],
                    color: lane_border_color,
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

            let mut size = h * 0.1;
            let mut screen_position = (x + w / 3.0 - size, y + h - size);
            let mut layout = wgpu_glyph::Layout::default()
                .h_align(wgpu_glyph::HorizontalAlign::Right)
                .v_align(wgpu_glyph::VerticalAlign::Bottom);

            if self.is_vertical_layout {
                size = w * 0.1;
                screen_position = (x + w / 2.0, h - (h / 5.0) + size);
                layout = wgpu_glyph::Layout::default_wrap()
                    .h_align(wgpu_glyph::HorizontalAlign::Center)
                    .v_align(wgpu_glyph::VerticalAlign::Top)
            }

            brush.queue(Section {
                screen_position,
                text: vec![wgpu_glyph::Text::new(lane.label().to_uppercase().as_str())
                    .with_color([1.0, 1.0, 1.0, 0.2])
                    .with_scale(size)],
                bounds: (w, h),
                layout,
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
