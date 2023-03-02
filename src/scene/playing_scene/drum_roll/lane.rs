use crate::{
    utils::{Point, Size},
};
use neothesia_pipelines::quad::QuadInstance;
use wgpu_jumpstart::Color;

pub struct Lane {
    pub(super) pos: Point<f32>,
    pub(super) size: Size<f32>,
    pub(super) note_id: u8,

    pressed_by_user: bool,
}

impl Lane {
    pub fn new() -> Self {

        Self {
            pos: Default::default(),
            size: Default::default(),
            note_id: 0,

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
        match self.note_id {
            27 => "High Q (GM2)",
            28 => "Slap (GM2)",
            29 => "Scratch Push (GM2)",
            30 => "Scratch Pull (GM2)",
            31 => "Sticks (GM2)",
            32 => "Square Click (GM2)",
            33 => "Metronome Click (GM2)",
            34 => "Metronome Bell (GM2)",
            35 => "Bass Drum 2",
            36 => "Bass Drum 1",
            37 => "Side Stick",
            38 => "Snare Drum 1",
            39 => "Hand Clap",
            40 => "Snare Drum 2",
            41 => "Low Tom 2",
            42 => "Closed Hi-hat",
            43 => "Low Tom 1",
            44 => "Pedal Hi-hat",
            45 => "Mid Tom 2",
            46 => "Open Hi-hat",
            47 => "Mid Tom 1",
            48 => "High Tom 2",
            49 => "Crash Cymbal 1",
            50 => "High Tom 1",
            51 => "Ride Cymbal 1",
            52 => "Chinese Cymbal",
            53 => "Ride Bell",
            54 => "Tambourine",
            55 => "Splash Cymbal",
            56 => "Cowbell",
            57 => "Crash Cymbal 2",
            58 => "Vibra Slap",
            59 => "Ride Cymbal 2",
            60 => "High Bongo",
            61 => "Low Bongo",
            62 => "Mute High Conga",
            63 => "Open High Conga",
            64 => "Low Conga",
            65 => "High Timbale",
            66 => "Low Timbale",
            67 => "High Agogo",
            68 => "Low Agogo",
            69 => "Cabasa",
            70 => "Maracas",
            71 => "Short Whistle",
            72 => "Long Whistle",
            73 => "Short Guiro",
            74 => "Long Guiro",
            75 => "Claves",
            76 => "High Wood Block",
            77 => "Low Wood Block",
            78 => "Mute Cuica",
            79 => "Open Cuica",
            80 => "Mute Triangle",
            81 => "Open Triangle",
            82 => "Shaker (GM2)",
            83 => "Jingle Bell (GM2)",
            84 => "Belltree (GM2)",
            85 => "Castanets (GM2)",
            86 => "Mute Surdo (GM2)",
            87 => "Open Surdo (GM2)",
            _ => "Unknown"
        }
    }
}

impl From<&Lane> for QuadInstance {
    fn from(lane: &Lane) -> QuadInstance {
        let color = if lane.pressed_by_user {
            let v = 0.5;
            Color::new(v, v, v, 0.0)
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
