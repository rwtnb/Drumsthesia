use crate::utils::{Point, Size};
use neothesia_pipelines::quad::QuadInstance;
use wgpu_jumpstart::Color;

pub struct Lane {
    pub pos: Point<f32>,
    pub size: Size<f32>,
    pub notes: [u8; 2],

    pressed_by_user: bool,
}

impl Lane {
    pub fn new(id: usize) -> Self {
        let notes = match id {
            0 => [49, 57],
            1 => [51, 59],
            2 => [53, 53],
            3 => [55, 55],
            4 => [52, 52],
            5 => [50, 48],
            6 => [47, 45],
            7 => [43, 41],
            8 => [46, 46],
            9 => [42, 42],
            10 => [38, 40],
            11 => [44, 44],
            12 => [36, 35],
            _ => panic!("invalid lane id"),
        };

        Self {
            pos: Default::default(),
            size: Default::default(),
            notes,

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
        match self.notes[0] {
            49 => "Crash Cymbal",
            57 => "Crash Cymbal",
            51 => "Ride Cymbal",
            59 => "Ride Cymbal",
            53 => "Ride Bell",
            55 => "Splash Cymbal",
            52 => "Chinese Cymbal",
            50 => "High Tom",
            48 => "High Tom",
            47 => "Mid Tom",
            45 => "Mid Tom",
            43 => "Low Tom",
            41 => "Low Tom",
            46 => "Open Hi-hat",
            42 => "Closed Hi-hat",
            38 => "Snare Drum",
            40 => "Snare Drum",
            44 => "Pedal Hi-hat",
            36 => "Bass Drum",
            35 => "Bass Drum",
            _ => "Unknown",
        }
    }
}

impl From<&Lane> for QuadInstance {
    fn from(lane: &Lane) -> QuadInstance {
        let color = if lane.pressed_by_user {
            Color::new(1.0, 1.0, 1.0, 0.05)
        } else {
            Color::new(0.0, 0.0, 0.0, 0.0)
        };

        QuadInstance {
            position: lane.pos.into(),
            size: lane.size.into(),
            color: color.into_linear_rgba(),
            border_radius: [0.0, 0.0, 0.0, 0.0],
        }
    }
}
