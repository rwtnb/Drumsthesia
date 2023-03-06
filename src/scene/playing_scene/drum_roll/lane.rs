use crate::utils::{Point, Size};
use crate::scene::playing_scene::midi_mapping::{MidiMapping, get_midi_mapping};
use neothesia_pipelines::quad::QuadInstance;
use wgpu_jumpstart::Color;

pub struct Lane {
    pub pos: Point<f32>,
    pub size: Size<f32>,
    pub mapping: MidiMapping,
    pressed_by_user: bool,
}

impl Lane {
    pub fn new(mapping: MidiMapping) -> Self {
        Self {
            pos: Default::default(),
            size: Default::default(),
            mapping,
            pressed_by_user: false,
        }
    }

    pub fn set_pressed_by_user(&mut self, is: bool) {
        self.pressed_by_user = is;
    }

    pub fn x_position(&self) -> f32 {
        self.pos.x
    }

    pub fn y_position(&self) -> f32 {
        self.pos.y
    }

    pub fn width(&self) -> f32 {
        self.size.w
    }

    pub fn height(&self) -> f32 {
        self.size.h
    }

    pub fn label(&self) -> &str {
        self.mapping.name
    }
}

impl From<&Lane> for QuadInstance {
    fn from(lane: &Lane) -> QuadInstance {
        let color = if lane.pressed_by_user {
            Color::new(0.2, 0.2, 0.2, 1.0)
        } else {
            Color::new(0.1, 0.1, 0.1, 0.8)
        };

        QuadInstance {
            position: lane.pos.into(),
            size: lane.size.into(),
            color: color.into_linear_rgba(),
            border_radius: [0.0, 0.0, 0.0, 0.0],
        }
    }
}
