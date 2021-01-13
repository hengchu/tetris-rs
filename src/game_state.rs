use lazy_static::lazy_static;
use std::collections::HashMap;
use std::convert::TryFrom;
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

    pub fn tick() {}
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
}
