mod game_state;
mod ui;

extern crate tui;

use game_state::{Event, Tetris};
use ui::*;

use std::io;
use std::sync::mpsc;
use std::thread;
use termion::event::Key;
use termion::input::TermRead;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::Terminal;

pub enum Iteration {
    /// A key press event to be handled
    Event(Event),
    /// A clock tick
    Tick,
}

struct Driver {
    rx: mpsc::Receiver<Iteration>,
    input_thread: thread::JoinHandle<()>,
    tick_thread: thread::JoinHandle<()>,
}

impl Driver {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        let input_thread = {
            let tx = tx.clone();
            thread::spawn(move || {
                let stdin = io::stdin();
                for evt in stdin.keys() {
                    match evt {
                        Ok(Key::Char('a')) => tx.send(Iteration::Event(Event::Left)).unwrap(),
                        Ok(Key::Char('d')) => tx.send(Iteration::Event(Event::Right)).unwrap(),
                        Ok(Key::Char('q')) => {
                            tx.send(Iteration::Event(Event::CounterClock)).unwrap()
                        }
                        Ok(Key::Char('e')) => tx.send(Iteration::Event(Event::Clock)).unwrap(),
                        Ok(Key::Esc) => std::process::exit(0),
                        _ => (),
                    }
                }
            })
        };
        let tick_thread = {
            thread::spawn(move || loop {
                tx.send(Iteration::Tick).unwrap();
                thread::sleep(std::time::Duration::from_millis(1000 / 6));
            })
        };

        Self {
            rx,
            input_thread,
            tick_thread,
        }
    }

    fn next(&self) -> Iteration {
        self.rx.recv().unwrap()
    }
}

fn main() -> Result<(), io::Error> {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut tetris = Tetris::new();
    let driver = Driver::new();

    loop {
        match driver.next() {
            Iteration::Tick => {
                if !tetris.tick() {
                    break;
                }
            }
            Iteration::Event(evt) => tetris.event(evt),
        }

        let render_grid = GridWidget(&tetris.grid());
        terminal
            .draw(|f| {
                let size = f.size();
                f.render_widget(render_grid, size);
            })
            .unwrap();
    }

    Ok(())
}
