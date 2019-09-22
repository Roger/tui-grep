mod event;
// mod event_cross;
// extern crate nix;

use std::io;
use termion::clear;
use termion::event::Key;
use termion::is_tty;
use termion::raw::IntoRawMode;
use termion::raw::RawTerminal;
use tui::backend::TermionBackend;
use tui::layout::{Constraint, Direction, Layout};
use tui::widgets::{Block, Borders, List, Paragraph, SelectableList, Text, Widget};
use tui::Terminal;

use fixed_vec_deque::FixedVecDeque;

use event::{Event, Events};

type TerminalResult = Result<Terminal<TermionBackend<RawTerminal<std::io::Stdout>>>, io::Error>;

fn get_terminal() -> TerminalResult {
    let stdout = io::stdout().into_raw_mode()?;
    let backend = TermionBackend::new(stdout);
    Ok(Terminal::new(backend)?)
}

#[derive(Default)]
struct Log {
    data: String,
}

fn main() -> Result<(), io::Error> {
    let stdin = io::stdin();
    if is_tty(&stdin) {
        panic!("Missing input");
    }

    let rex = regex::Regex::new(r"(.*)([A-Z]*)(.*)").unwrap();

    // Clear the screen.
    println!("{}", clear::All);

    let mut terminal = get_terminal()?;
    let events = Events::new();
    // let mut logs: Vec<String> = vec![];
    let mut logs = FixedVecDeque::<[Log; 0x10000]>::new();

    loop {
        match events.next().unwrap() {
            Event::Key(input) => match input {
                Key::Char('q') => {
                    break;
                }
                // Key::Left => {
                //     app.selected = None;
                // }
                // Key::Down => {
                //     app.selected = if let Some(selected) = app.selected {
                //         if selected >= app.items.len() - 1 {
                //             Some(0)
                //         } else {
                //             Some(selected + 1)
                //         }
                //     } else {
                //         Some(0)
                //     }
                // }
                // Key::Up => {
                //     app.selected = if let Some(selected) = app.selected {
                //         if selected > 0 {
                //             Some(selected - 1)
                //         } else {
                //             Some(app.items.len() - 1)
                //         }
                //     } else {
                //         Some(0)
                //     }
                // }
                _ => {}
            },
            Event::Input(line) => {
                logs.push_back().data = line;
            }
            Event::Tick => {
                // dbg!("Tick");
                // app.advance();
                terminal
                    .draw(|mut f| {
                        let win_size = f.size();
                        let chunks = Layout::default()
                            .direction(Direction::Vertical)
                            // .margin(1)
                            .constraints([Constraint::Percentage(100)].as_ref())
                            .split(win_size);

                        let logs_end = logs.len();
                        let mut logs_start: i32 = logs_end as i32 - win_size.height as i32;
                        if logs_start < 0 {
                            logs_start = 0;
                        }

                        let logs_formated = logs
                            .iter()
                            .skip(logs_start as usize)
                            .map(|log| {
                                Text::raw(&log.data)
                            });
                        List::new(logs_formated)
                            .block(Block::default())
                            .render(&mut f, chunks[0]);
                    })
                    .expect("Can't draw");
            }
        }
    }

    Ok(())
}
