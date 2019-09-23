mod event;
mod event_cross;

use regex::Regex;
use std::io::{self, stdin, stdout, Write};
use termion::event::Key;
use termion::is_tty;
use termion::raw::IntoRawMode;
use termion::{clear, terminal_size};
use termion::{color, style};

use fixed_vec_deque::FixedVecDeque;

use event_cross::{Event, Events};

#[derive(Debug, Default)]
struct Log {
    data: String,
}

#[derive(Debug)]
enum Mode {
    Normal,
    Command,
}

#[derive(Debug)]
struct State {
    mode: Mode,
    lines: FixedVecDeque<[Log; 0x10000]>,
    filter: String,
    filter_re: Option<Regex>,
    highlight: String,
    highlight_re: Option<Regex>,
}

impl State {
    fn new() -> Self {
        Self {
            mode: Mode::Normal,
            lines: FixedVecDeque::<[Log; 0x10000]>::new(),
            filter: "".to_string(),
            filter_re: None,
            highlight: "".to_string(),
            highlight_re: None,
        }
    }

    fn add_line(self: &mut Self, line: String) {
        self.lines.push_back().data = line;
    }
}

fn main() -> Result<(), io::Error> {
    let stdin = stdin();
    if is_tty(&stdin) {
        panic!("Missing input");
    }

    let mut stdout = stdout().into_raw_mode().unwrap();

    // Clear the screen.
    println!("{}{}", clear::All, termion::cursor::Hide);

    let events = Events::new();
    let mut state = State::new();

    loop {
        match events.next() {
            Event::Key(input) => match state.mode {
                // Command mode
                Mode::Normal => match input {
                    Key::Char('q') => {
                        break;
                    }
                    Key::Char(':') => {
                        state.mode = Mode::Command;
                    }
                    _ => {}
                },
                // Handle writing regex
                Mode::Command => match input {
                    Key::Char('q') => {
                        break;
                    }
                    Key::Backspace => {
                        state.filter.pop();
                        if state.filter.is_empty() {
                            state.filter_re = None;
                        } else if let Ok(rex) = regex::Regex::new(&state.filter) {
                            state.filter_re = Some(rex);
                        }
                    }
                    Key::Char(c) => {
                        state.filter.push(c);

                        if state.filter.is_empty() {
                            state.filter_re = None;
                        } else if let Ok(rex) = regex::Regex::new(&state.filter) {
                            state.filter_re = Some(rex);
                        }
                    }
                    _ => {}
                },
            },
            Event::Input(line) => {
                state.add_line(line);
            }
            Event::Tick => {
                let (width, height) = terminal_size().unwrap();
                let logs_formated: Vec<String> = state
                    .lines
                    .iter()
                    .filter(|log| {
                        if let Some(rex) = state.filter_re.as_ref() {
                            return rex.is_match(&log.data);
                        }
                        true
                    })
                    .map(|log| {
                        let mut data = log.data.clone();
                        if data.len() as u16 > width {
                            let t = &log.data[0..width as usize];
                            data = t.to_string();
                        }

                        if let Some(rex) = state.filter_re.as_ref() {
                            let replacer = format!("{}$0{}", color::Bg(color::Red), style::Reset);
                            return rex.replace_all(&data, replacer.as_str()).to_string();
                        }

                        data
                    })
                    .collect();

                let logs_end = logs_formated.len();
                let mut logs_start: i32 = logs_end as i32 - height as i32;
                if logs_start < 0 {
                    logs_start = 0;
                }

                // let logs_formated: Vec<String> = state

                let mut out = "".to_string();
                for (number, line) in logs_formated[logs_start as usize..].iter().enumerate() {
                    let line_end = line.len();

                    // Remove leftover
                    out.push_str(
                        format!(
                            "{}{}",
                            termion::cursor::Goto(line_end as u16, number as u16),
                            clear::UntilNewline
                        )
                        .as_ref(),
                    );

                    out.push_str(
                        format!("{}{}", termion::cursor::Goto(1, number as u16), line).as_ref(),
                    );
                }

                out.push_str(
                    format!(
                        "{}{}",
                        termion::cursor::Goto(0, logs_formated.len() as u16),
                        clear::AfterCursor,
                    )
                    .as_ref(),
                );

                write!(stdout, "{}", out).unwrap();
                stdout.flush().unwrap();

                write!(
                    stdout,
                    "{}:filter {}{}",
                    termion::cursor::Goto(0, height),
                    state.filter,
                    clear::UntilNewline,
                )
                .unwrap();
                stdout.flush().unwrap();
            }
        }
    }

    println!("{}", termion::cursor::Show);

    Ok(())
}
