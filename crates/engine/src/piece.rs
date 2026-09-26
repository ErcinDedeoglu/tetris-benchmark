//! The seven standard tetrominoes with four clockwise rotation states each.

pub const SHAPE_NAMES: [&str; 7] = ["I", "O", "T", "S", "Z", "J", "L"];

const BASE_CELLS: [&[(usize, usize)]; 7] = [
    &[(0, 0), (0, 1), (0, 2), (0, 3)], // I
    &[(0, 0), (0, 1), (1, 0), (1, 1)], // O
    &[(0, 1), (1, 0), (1, 1), (1, 2)], // T
    &[(0, 1), (0, 2), (1, 0), (1, 1)], // S
    &[(0, 0), (0, 1), (1, 1), (1, 2)], // Z
    &[(0, 0), (1, 0), (1, 1), (1, 2)], // J
    &[(0, 2), (1, 0), (1, 1), (1, 2)], // L
];

/// Rotating a shape clockwise inside its bounding box: (r, c) -> (c, H-1-r).
fn rotate_cells(cells: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let height = cells.iter().map(|&(r, _)| r).max().unwrap() + 1;
    cells.iter().map(|&(r, c)| (c, height - 1 - r)).collect()
}

/// Normalize a cell set so its top-left cell sits at (0, 0).
fn normalize(cells: &[(usize, usize)]) -> Vec<(usize, usize)> {
    let min_r = cells.iter().map(|&(r, _)| r).min().unwrap();
    let min_c = cells.iter().map(|&(_, c)| c).min().unwrap();
    let mut out: Vec<(usize, usize)> = cells.iter().map(|&(r, c)| (r - min_r, c - min_c)).collect();
    out.sort_unstable();
    out
}

/// One tetromino shape: its name and its four clockwise rotation states.
pub struct Shape {
    pub name: &'static str,
    pub states: Vec<Vec<(i32, i32)>>,
}

impl Shape {
    fn from_base(base: &'static [(usize, usize)], name: &'static str) -> Self {
        let mut states = Vec::new();
        let mut current: Vec<(usize, usize)> = base.to_vec();
        for _ in 0..4 {
            let normalized = normalize(&current);
            states.push(normalized.iter().map(|&(r, c)| (r as i32, c as i32)).collect());
            current = rotate_cells(&current);
        }
        Shape { name, states }
    }

    pub fn width(&self, state: usize) -> i32 {
        self.states[state].iter().map(|&(_, c)| c).max().unwrap() + 1
    }
}

pub fn all_shapes() -> Vec<Shape> {
    SHAPE_NAMES
        .iter()
        .zip(BASE_CELLS.iter())
        .map(|(&name, &base)| Shape::from_base(base, name))
        .collect()
}

/// A live piece: which shape, which rotation state, and the anchor cell
/// (top-left of the current state's bounding box).
#[derive(Clone, Copy)]
pub struct Piece {
    pub shape: usize,
    pub state: usize,
    pub row: i32,
    pub col: i32,
}

impl Piece {
    pub fn spawn(shape: usize, board_width: i32) -> Self {
        let shapes = all_shapes();
        let width = shapes[shape].width(0);
        Piece {
            shape,
            state: 0,
            row: 0,
            col: (board_width - width) / 2,
        }
    }

    pub fn cells(&self) -> Vec<(i32, i32)> {
        all_shapes()[self.shape].states[self.state % 4]
            .iter()
            .map(|&(r, c)| (self.row + r, self.col + c))
            .collect()
    }

    pub fn shape_name(&self) -> &'static str {
        SHAPE_NAMES[self.shape]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn seven_shapes_with_four_states() {
        let shapes = all_shapes();
        assert_eq!(shapes.len(), 7);
        for s in &shapes {
            assert_eq!(s.states.len(), 4, "{}", s.name);
            for st in &s.states {
                assert_eq!(st.len(), 4, "{}", s.name);
            }
        }
    }

    #[test]
    fn i_spawns_centered_on_ten_wide_board() {
        let p = Piece::spawn(0, 10);
        let cells = p.cells();
        assert_eq!(cells, vec![(0, 3), (0, 4), (0, 5), (0, 6)]);
    }

    #[test]
    fn o_spawns_at_columns_four_and_five() {
        let p = Piece::spawn(1, 10);
        assert_eq!(p.cells(), vec![(0, 4), (0, 5), (1, 4), (1, 5)]);
    }

    #[test]
    fn rotation_states_stay_within_bounds() {
        let shapes = all_shapes();
        for s in &shapes {
            for st in &s.states {
                for &(r, c) in st {
                    assert!((0..4).contains(&r), "{}", s.name);
                    assert!((0..4).contains(&c), "{}", s.name);
                }
            }
        }
    }
}
