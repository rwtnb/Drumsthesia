pub mod range;

pub struct Keybard {
    pub keys: Vec<Key>,
    pub neutral_width: f32,
    pub neutral_height: f32,
}

#[derive(Debug)]
pub enum KeyKind {
    Neutral,
}

#[derive(Debug)]
pub struct Key {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    kind: KeyKind,
    note_id: u8,
}

impl Key {
    pub fn x(&self) -> f32 {
        self.x
    }

    pub fn y(&self) -> f32 {
        self.y
    }

    pub fn size(&self) -> (f32, f32) {
        (self.width, self.height)
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn height(&self) -> f32 {
        self.height
    }

    pub fn kind(&self) -> &KeyKind {
        &self.kind
    }

    pub fn note_id(&self) -> u8 {
        self.note_id
    }
}

struct Sizing {
    neutral_width: f32,
    neutral_height: f32,
}

pub fn standard_88_keys(neutral_width: f32, neutral_height: f32) -> Keybard {
    let sizing = Sizing {
        neutral_width,
        neutral_height,
    };

    let mut offset = 0.0;
    let mut keys = Vec::<Key>::new();
    for note_id in 27..53 {
        keys.push(Key {
            x: 0.0,
            y: offset,
            width: sizing.neutral_width,
            height: sizing.neutral_height,
            kind: KeyKind::Neutral,
            note_id,
        });
        offset += sizing.neutral_height;
    }

    Keybard {
        keys,
        neutral_width,
        neutral_height,
    }
}
