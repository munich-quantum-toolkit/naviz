use crate::{Color, Position};

/// Dynamic state (i.e., often changes)
#[derive(Clone, Debug)]
pub struct State {
    /// The atoms
    pub atoms: Vec<AtomState>,
    // The time (full string to draw; i.e., with time prefix)
    pub time: String,
}

#[derive(Clone, Debug)]
pub struct AtomState {
    /// The position of this atom
    pub position: Position,
    /// The radius of this atom
    pub size: f32,
    /// The color of this atom
    pub color: Color,
    /// Whether this atom is currently shuttling
    pub shuttle: bool,
    /// The label to draw on this atom
    pub label: String,
}

impl State {
    /// An example [State]
    pub fn example() -> Self {
        Self {
            atoms: (0..=16)
                .map(|i| (i, i as f32))
                .map(|(idx, i)| (idx, i * 2647. % 97., i * 6373. % 113., i * 5407. % 7. > 3.5))
                .map(|(idx, x, y, s)| AtomState {
                    position: (x, y),
                    size: 3.,
                    color: [255, 128, 32, 255],
                    shuttle: s,
                    label: format!("{idx}"),
                })
                .collect(),
            time: "Time: 42 us".to_owned(),
        }
    }
}
