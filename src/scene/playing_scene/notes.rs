use crate::config::PlayingSceneLayout;
use crate::target::Target;
use crate::TransformUniform;
use crate::Uniform;
use neothesia_pipelines::waterfall::{NoteInstance, WaterfallPipeline};
use wgpu_jumpstart::Color;


use super::drum_roll::Lane;

pub struct Notes {
    notes_pipeline: WaterfallPipeline,
    is_vertical_layout: bool
}

impl Notes {
    pub fn new(target: &mut Target, lanes: &[Lane]) -> Self {
        let is_vertical_layout = target.config.layout == PlayingSceneLayout::Vertical;
        let notes_pipeline = WaterfallPipeline::new(
            &target.gpu,
            &target.transform_uniform,
            target.midi_file.as_ref().unwrap().merged_track.notes.len(),
            is_vertical_layout
        );
        let mut notes = Self { notes_pipeline, is_vertical_layout };
        notes.resize(target, lanes);
        notes
    }

    pub fn resize(&mut self, target: &mut Target, lanes: &[Lane]) {
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
                    let note_duration = note.duration.as_secs_f32() * 4.0;
                    let mut note_h = f32::min(lane.size.h * 0.6, 100.0);
                    let note_w = note_duration * note_h;
                    let mut note_w = if note_w <= note_h * 0.6 {
                        note_h * 0.5
                    } else if note_w <= note_h * 0.8 {
                        note_h * 0.75
                    } else {
                        note_h
                    };

                    let mut x = note.start.as_secs_f32();
                    let mut y = lane.pos.y + (lane.size.h / 2.0) - (note_h / 2.0);

                    if self.is_vertical_layout {
                        note_w = f32::min(lane.size.w * 0.6, 100.0);
                        note_h = note_duration * note_w;
                        note_h = if note_h <= note_w * 0.6 {
                            note_w * 0.5
                        } else if note_h <= note_w * 0.8 {
                            note_w * 0.75
                        } else {
                            note_w
                        };

                        x = lane.pos.x + (lane.size.w / 2.0) - (note_w / 2.0);
                        y = note.start.as_secs_f32();
                    }           

                    instances.push(NoteInstance {
                        position: [x, y],
                        size: [note_w, note_h],
                        color: color.into_linear_rgb(),
                        radius: note_h * 0.1,
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
