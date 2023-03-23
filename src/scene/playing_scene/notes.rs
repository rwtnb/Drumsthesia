use crate::config::PlayingSceneLayout;
use crate::target::Target;
use crate::TransformUniform;
use crate::Uniform;
use neothesia_pipelines::waterfall::{NoteInstance, WaterfallPipeline};
use wgpu_jumpstart::Color;

use super::drum_roll::Lane;

pub struct Notes {
    notes_pipeline: WaterfallPipeline,
    is_vertical_layout: bool,
}

impl Notes {
    pub fn new(target: &mut Target, lanes: &[Lane]) -> Self {
        let is_vertical_layout = target.config.layout == PlayingSceneLayout::Vertical;
        let notes_pipeline = WaterfallPipeline::new(
            &target.gpu,
            &target.transform_uniform,
            target.midi_file.as_ref().unwrap().merged_track.notes.len(),
            is_vertical_layout,
        );
        let mut notes = Self {
            notes_pipeline,
            is_vertical_layout,
        };
        notes.resize(target, lanes);
        notes
    }

    pub fn resize(&mut self, target: &mut Target, lanes: &[Lane]) {
        let midi = &target.midi_file.as_ref().unwrap();
        let mut instances = Vec::new();

        if !self.is_vertical_layout {
            let lane = lanes.first().unwrap();
            let metronome_grid = [0.05, 0.05, 0.05];
            let metronome_events = midi
                .merged_track
                .events
                .iter()
                .filter(|n| n.channel == 9 && n.track_id == 99);

            for (i, event) in metronome_events.clone().enumerate() {
                let ref_event = if i == 0 {
                    metronome_events.clone().next()
                } else {
                    metronome_events.clone().nth(i - 1)
                };

                if ref_event.is_none() {
                    log::error!("failed to find metronome events");
                    continue;
                }

                let ref_event = ref_event.unwrap();

                let grid_size = if i == 0 {
                    (ref_event.timestamp - event.timestamp).as_secs_f32()
                } else {
                    (event.timestamp - ref_event.timestamp).as_secs_f32()
                };

                let x = event.timestamp.as_secs_f32() - grid_size * 2.0;
                let h = lanes.len() as f32 * lane.size.h;

                instances.push(NoteInstance {
                    position: [x, lane.pos.y],
                    size: [1.0, h],
                    color: metronome_grid,
                    radius: 0.0,
                });
            }
        }

        for note in midi
            .merged_track
            .notes
            .iter()
            .filter(|n| n.channel == 9 && n.track_id != 99)
        {
            match lanes.iter().find(|i| i.mapping.accept_note(note.note)) {
                None => {
                    println!("missing mapping for note {}", note.note);
                }

                Some(lane) => {
                    let color: Color = lane.mapping.color.into();
                    let note_duration = note.duration.as_secs_f32() * 5.0;
                    let mut note_h = f32::min(lane.size.h * 0.6, 100.0);
                    let mut note_w = f32::min(note_duration * note_h, note_h);

                    let mut x = note.start.as_secs_f32();
                    let mut y = lane.pos.y + (lane.size.h / 2.0) - (note_h / 2.0);

                    if self.is_vertical_layout {
                        note_w = f32::min(lane.size.w * 0.6, 100.0);
                        note_h = f32::min(note_duration * note_w, note_w);

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
