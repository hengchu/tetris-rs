use super::game_state::{Grid, NCOLS, NROWS};
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::style::Color;
use tui::widgets::Widget;

/// A newtype wrapper around a grid for rendering as tui widget.
pub struct GridWidget<'a>(pub &'a Grid);

impl<'a> Widget for GridWidget<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if (area.width as usize) < NCOLS || (area.height as usize) < NROWS {
            panic!("Terminal UI area too small!");
        }

        if (buf.area.width as usize) < NCOLS || (buf.area.height as usize) < NROWS {
            panic!("Terminal UI buffer area too small!");
        }

        buf.reset();
        let square = b"\xe2\x96\xa1";
        let square_str = std::str::from_utf8(square).expect("square is invalid");

        for row in 0..NROWS {
            for col in 0..NCOLS {
                let idx = buf.index_of(col as u16, row as u16);
                let cell_mut = &mut buf.content[idx];
                if self.0[row][col] == 1 {
                    cell_mut
                        .set_symbol(square_str)
                        .set_fg(Color::White)
                        .set_bg(Color::Black);
                } else {
                    cell_mut.set_bg(Color::Black);
                }
            }
        }
    }
}
