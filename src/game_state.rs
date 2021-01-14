use lazy_static::lazy_static;
use std::cmp::{max, min};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fmt;

type Offsets = (i32, i32);

/// Assuming each piece pseudo-occupies a 4x4 square, then these 4
/// offsets gives the cells that are actually occupied in that 4x4
/// square. The 1st one is un-rotated, 2nd one rotated clockwise 90
/// degrees, 3rd one rotated 180 degrees, and 4th one rotated 270
/// degrees.
type Offsets4 = [Offsets; 4];

lazy_static! {
    static ref ROTATION_OFFSETS: HashMap<Piece, [Offsets4; 4]> = {
        let mut data = HashMap::new();
        // ##
        // ##
        data.insert(Piece::O,
                    [[(0, 0), (0, 1), (1, 0), (1, 1)],
                     [(0, 0), (0, 1), (1, 0), (1, 1)],
                     [(0, 0), (0, 1), (1, 0), (1, 1)],
                     [(0, 0), (0, 1), (1, 0), (1, 1)]]);

        // #        ##
        // #   ###   #    #
        // ##, #  ,  #, ###
        data.insert(Piece::L,
                    [[(0, 0), (1, 0), (2, 0), (2, 1)],
                     [(1, 0), (1, 1), (1, 2), (2, 0)],
                     [(0, 0), (0, 1), (1, 1), (2, 1)],
                     [(2, 0), (2, 1), (2, 2), (1, 1)]]);

        //  #       ##
        //  #  #    #   ###
        // ##, ###, # ,   #
        data.insert(Piece::J,
                    [[(2, 0), (2, 1), (0, 1), (1, 1)],
                     [(1, 0), (2, 0), (2, 1), (2, 2)],
                     [(0, 0), (1, 0), (2, 0), (0, 1)],
                     [(1, 0), (1, 1), (1, 2), (2, 2)]]);

        // ###   #   #   #
        //  #   ##  ###  ##
        //    ,  #,    , #
        data.insert(Piece::T,
                    [[(0, 0), (0, 1), (0, 2), (1, 1)],
                     [(1, 0), (0, 1), (1, 1), (2, 1)],
                     [(1, 0), (0, 1), (1, 1), (1, 2)],
                     [(0, 0), (1, 0), (2, 0), (1, 1)]]);

        // ##     #   ##     #
        //  ##   ##    ##   ##
        //    ,  #  ,     , #
        data.insert(Piece::Z,
                    [[(0, 0), (0, 1), (1, 1), (1, 2)],
                     [(1, 0), (0, 1), (1, 1), (2, 0)],
                     [(0, 0), (0, 1), (1, 1), (1, 2)],
                     [(1, 0), (0, 1), (1, 1), (2, 0)]]);

        //  ##  #     ##   #
        // ##   ##   ##    ##
        //    ,  # ,     ,  #
        data.insert(Piece::S,
                    [[(1, 0), (0, 1), (1, 1), (0, 2)],
                     [(0, 0), (1, 0), (1, 1), (2, 1)],
                     [(1, 0), (0, 1), (1, 1), (0, 2)],
                     [(0, 0), (1, 0), (1, 1), (2, 1)]]);

        // #         #
        // #  #####  #  ####
        // #         #
        // #,      , #,
        data.insert(Piece::I,
                    [[(0, 0), (1, 0), (2, 0), (3, 0)],
                     [(1, 0), (1, 1), (1, 2), (1, 3)],
                     [(0, 0), (1, 0), (2, 0), (3, 0)],
                     [(1, 0), (1, 1), (1, 2), (1, 3)]]);

        data
    };
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Copy, Clone, Hash)]
pub enum Piece {
    O = 0,
    L,
    J,
    T,
    Z,
    S,
    I,
}

impl TryFrom<i32> for Piece {
    type Error = ();

    fn try_from(v: i32) -> Result<Self, Self::Error> {
        match v {
            0 => Ok(Self::O),
            1 => Ok(Self::L),
            2 => Ok(Self::J),
            3 => Ok(Self::T),
            4 => Ok(Self::Z),
            5 => Ok(Self::S),
            6 => Ok(Self::I),
            _ => Err(()),
        }
    }
}

const NCOLS: usize = 10;
const NROWS: usize = 20;
type Grid = [[i32; NCOLS]; NROWS];

#[derive(Clone, PartialEq, Eq)]
pub struct Tetris {
    /// The current NROWS x NCOLS tetris
    pub(super) grid: Grid,
    /// The type of current falling piece
    pub(super) piece: Piece,
    /// The rotation state of current falling piece
    pub(super) rotation: i32,
    /// The anchor row coordinate of falling piece
    pub(super) anchor_row: i32,
    /// The anchor col coordinate of falling piece
    pub(super) anchor_col: i32,
}

impl fmt::Debug for Tetris {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("========TETRIS========\n")?;
        f.write_fmt(format_args!(
            "Piece: {:?}, Rotation: {}, ARow: {}, ACol: {}\n",
            self.piece, self.rotation, self.anchor_row, self.anchor_col
        ))?;
        for row in 0usize..NROWS {
            let mut row_str = String::new();
            for col in 0usize..NCOLS {
                row_str += &self.grid[row][col].to_string();
            }
            row_str += "\n";
            f.write_str(row_str.as_str())?;
        }
        Ok(())
    }
}

/// Paint or clear the grid for the given piece with given rotation
/// and anchor location.
fn update(
    grid: &mut Grid,
    piece: Piece,
    rotation: i32,
    anchor_row: i32,
    anchor_col: i32,
    fill: bool,
) {
    let rotation_offsets: &[Offsets4; 4] = ROTATION_OFFSETS.get(&piece).unwrap();
    let offsets: &Offsets4 = &rotation_offsets[rotation as usize];
    for (off_row, off_col) in offsets.iter() {
        let row: usize = (anchor_row + off_row) as usize;
        let col: usize = (anchor_col + off_col) as usize;
        if fill {
            grid[row][col] = 1;
        } else {
            grid[row][col] = 0;
        }
    }
}

impl Tetris {
    /// Create a new tetris game state object.
    pub fn new() -> Self {
        let mut grid: Grid = [[0; 10]; 20];
        update(&mut grid, Piece::O, 0, 0, 4, true);
        Self {
            grid,
            piece: Piece::O,
            rotation: 0,
            anchor_row: 0,
            anchor_col: 4,
        }
    }

    /// Fetch all positions of the current falling piece.
    fn falling_piece_positions(&self) -> Vec<(i32, i32)> {
        // TODO: a length-4 slice is fine, and we avoid allocation.
        let mut results = Vec::new();
        let rotation_offsets: &[Offsets4; 4] = ROTATION_OFFSETS.get(&self.piece).unwrap();
        for (off_row, off_col) in rotation_offsets[self.rotation as usize].iter() {
            results.push((self.anchor_row + off_row, self.anchor_col + off_col));
        }
        results
    }

    /// Checks if a new piece at the given row, col, and rotation
    /// overlaps with any existing cells.
    fn fits(grid: &Grid, piece: Piece, row: i32, col: i32, rotation: i32) -> bool {
        let rotation_offsets: &[Offsets4; 4] = ROTATION_OFFSETS.get(&piece).unwrap();
        for (off_row, off_col) in rotation_offsets[rotation as usize].iter() {
            let this_row = off_row + row;
            let this_col = off_col + col;
            if grid[this_row as usize][this_col as usize] == 1 {
                return false;
            }
        }
        true
    }

    /// Tests whether current falling piece can drop one more unit or
    /// not.
    fn can_drop(&self) -> bool {
        let positions = self.falling_piece_positions();
        for (row, col) in positions.iter() {
            let next_row = row + 1;
            if next_row == NROWS as i32 {
                return false;
            }
            // if the cell 1 unit down is not part of the piece
            // itself, and that the cell is filled, then we cannot
            // drop further.
            if !positions.contains(&(next_row, *col))
                && self.grid[next_row as usize][*col as usize] == 1
            {
                return false;
            }
        }

        true
    }

    /// Shift everything above row down by 1.
    fn shift_down(&mut self, row: i32) {
        for r in (1..row as usize).rev() {
            for c in 0..NCOLS {
                self.grid[r][c] = self.grid[r - 1][c];
            }
        }
        for c in 0..NCOLS {
            self.grid[0][c] = 0;
        }
    }

    /// Simulate "gravity" for 1 unit of time. Returns true if the game can still continue
    /// otherwise returns false.
    pub fn tick(&mut self) -> bool {
        let should_continue: bool;
        // 1. if we can drop, then just drop
        // 2. if we cannot drop, then check if there are complete rows
        // 3. check if there is enough space for new piece
        if self.can_drop() {
            update(
                &mut self.grid,
                self.piece,
                self.rotation,
                self.anchor_row,
                self.anchor_col,
                false,
            );
            self.anchor_row += 1;
            update(
                &mut self.grid,
                self.piece,
                self.rotation,
                self.anchor_row,
                self.anchor_col,
                true,
            );
            should_continue = true;
        } else {
            // 2. check for complete rows
            let mut min_row: i32 = NROWS as i32;
            let mut max_row: i32 = 0;

            for (row, _) in self.falling_piece_positions().iter() {
                min_row = min(min_row, *row);
                max_row = max(max_row, *row);
            }

            // shift things down by 1 if there are complete rows.
            for row in min_row..=max_row {
                if self.grid[row as usize].iter().sum::<i32>() == NCOLS as i32 {
                    self.shift_down(row);
                }
            }

            let new_piece: Piece = ((self.piece as i32 + 1) % 7).try_into().unwrap();
            if Self::fits(&self.grid, new_piece, 0, 4, 0) {
                self.piece = new_piece;
                self.rotation = 0;
                self.anchor_row = 0;
                self.anchor_col = 4;
                should_continue = true;

                update(
                    &mut self.grid,
                    self.piece,
                    self.rotation,
                    self.anchor_row,
                    self.anchor_col,
                    true,
                );
            } else {
                should_continue = false;
            }
        }

        should_continue
    }

    pub fn event() {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init_tetris() {
        let t = Tetris::new();
        let repr = format!("{:?}", t);
        let expected = r#"========TETRIS========
Piece: O, Rotation: 0, ARow: 0, ACol: 4
0000110000
0000110000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
"#;
        assert_eq!(repr, expected);
    }

    #[test]
    fn test_tick() {
        let mut t = Tetris::new();
        t.tick();
        let repr = format!("{:?}", t);
        println!("{}", repr);
        let expected = r#"========TETRIS========
Piece: O, Rotation: 0, ARow: 1, ACol: 4
0000000000
0000110000
0000110000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
"#;
        assert_eq!(repr, expected);
    }

    #[test]
    fn test_tick_until_bottom() {
        let mut t = Tetris::new();
        for _ in 0..19 {
            assert!(t.tick());
        }
        let repr = format!("{:?}", t);
        let expected = r#"========TETRIS========
Piece: L, Rotation: 0, ARow: 0, ACol: 4
0000100000
0000100000
0000110000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000000000
0000110000
0000110000
"#;
        assert_eq!(repr, expected);
    }
}
