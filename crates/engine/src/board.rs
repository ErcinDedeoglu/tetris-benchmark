//! The 10 wide by 20 tall playfield.

pub const WIDTH: usize = 10;
pub const HEIGHT: usize = 20;

#[derive(Clone)]
pub struct Board {
    cells: [[bool; WIDTH]; HEIGHT],
}

impl Default for Board {
    fn default() -> Self {
        Self::new()
    }
}

impl Board {
    pub fn new() -> Self {
        Board {
            cells: [[false; WIDTH]; HEIGHT],
        }
    }

    pub fn filled(&self, row: usize, col: usize) -> bool {
        self.cells[row][col]
    }

    pub fn cells_free(&self, cells: &[(i32, i32)]) -> bool {
        cells.iter().all(|&(r, c)| {
            r >= 0
                && c >= 0
                && (r as usize) < HEIGHT
                && (c as usize) < WIDTH
                && !self.cells[r as usize][c as usize]
        })
    }

    pub fn set_cells(&mut self, cells: &[(i32, i32)]) {
        for &(r, c) in cells {
            if r >= 0 && c >= 0 {
                self.cells[r as usize][c as usize] = true;
            }
        }
    }

    /// Remove every full row, drop the rows above, return how many cleared.
    pub fn clear_full_rows(&mut self) -> usize {
        let mut cleared = 0;
        let mut row = 0;
        while row < HEIGHT {
            if self.cells[row].iter().all(|&b| b) {
                for r in (1..=row).rev() {
                    self.cells[r] = self.cells[r - 1];
                }
                self.cells[0] = [false; WIDTH];
                cleared += 1;
            } else {
                row += 1;
            }
        }
        cleared
    }

    /// Render the board as the protocol block: piece line, board frame,
    /// 20 rows of exactly 10 cells each.
    pub fn render(&self, piece_name: &str, piece_cells: &[(i32, i32)]) -> String {
        let mut out = String::new();
        out.push_str("piece ");
        out.push_str(piece_name);
        out.push('\n');
        out.push_str("board begin\n");
        for r in 0..HEIGHT {
            let mut line = String::with_capacity(WIDTH);
            for c in 0..WIDTH {
                let ch = if self.cells[r][c] {
                    '#'
                } else if piece_cells.contains(&(r as i32, c as i32)) {
                    '@'
                } else {
                    '.'
                };
                line.push(ch);
            }
            out.push_str(&line);
            out.push('\n');
        }
        out.push_str("board end\n");
        out
    }

    /// How many rows the stack occupies, measured from the floor to its
    /// highest filled cell; 0 for an empty board.
    pub fn stack_height(&self) -> usize {
        (0..HEIGHT)
            .find(|&r| self.cells[r].iter().any(|&b| b))
            .map(|r| HEIGHT - r)
            .unwrap_or(0)
    }
}

/// Line-clear scores: 1 -> 100, 2 -> 300, 3 -> 500, 4 -> 800.
pub fn score_for_lines(cleared: usize) -> u64 {
    match cleared {
        1 => 100,
        2 => 300,
        3 => 500,
        4 => 800,
        _ => 0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_is_twenty_rows_of_ten() {
        let b = Board::new();
        let text = b.render("I", &[(0, 3), (0, 4), (0, 5), (0, 6)]);
        let lines: Vec<&str> = text.lines().collect();
        assert_eq!(lines[0], "piece I");
        assert_eq!(lines[1], "board begin");
        assert_eq!(lines[22], "board end");
        assert_eq!(lines.len(), 23);
        for row in &lines[2..22] {
            assert_eq!(row.len(), 10);
        }
        assert_eq!(lines[2], "...@@@@...");
        assert_eq!(lines[3], "..........");
    }

    #[test]
    fn clears_full_rows_only() {
        let mut b = Board::new();
        b.set_cells(&[
            (19, 0),
            (19, 1),
            (19, 2),
            (19, 3),
            (19, 4),
            (19, 5),
            (19, 6),
            (19, 7),
            (19, 8),
            (19, 9),
            (18, 0),
            (17, 5),
        ]);
        assert_eq!(b.clear_full_rows(), 1);
        assert!(b.filled(19, 0)); // dropped from row 18
        assert!(b.filled(18, 5)); // fell from row 17
        assert!(!b.filled(19, 5)); // the hole came down with it
        assert_eq!(b.stack_height(), 2);
    }

    #[test]
    fn scoring_table() {
        assert_eq!(score_for_lines(0), 0);
        assert_eq!(score_for_lines(1), 100);
        assert_eq!(score_for_lines(2), 300);
        assert_eq!(score_for_lines(3), 500);
        assert_eq!(score_for_lines(4), 800);
    }
}
