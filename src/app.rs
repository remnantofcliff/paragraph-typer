use std::{
    io::{stdout, StdoutLock, Write},
    time::Duration,
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyModifiers},
    queue,
    style::{Color, SetBackgroundColor, SetForegroundColor},
    terminal::{self, ClearType},
};

use crate::{
    time::Timer,
    utils::{self, count_spaces, word_wrap_keep_space},
};

/// How many lines do the bottom info texts take
const INFO_HEIGHT: usize = 3;

/// Used for initialization of a variable in App::draw_text()
static EMPTY_STRING: String = String::new();

pub struct App<'a> {
    current_line: usize,
    terminal_height: u16,
    text: &'a str,
    timer: Timer,
    typed: String,
    wrapped: Box<[&'a str]>,
}

impl<'a> App<'a> {
    pub fn new(text: &'a str) -> Self {
        let (wrapped, terminal_height) = {
            let (width, height) = terminal::size().unwrap();
            (word_wrap_keep_space(text, width as usize), height)
        };
        let typed = String::with_capacity(text.len());
        let timer = Timer::new();
        let current_line = 0;
        Self {
            current_line,
            terminal_height,
            text,
            timer,
            typed,
            wrapped,
        }
    }

    pub fn draw(&mut self, stdout: &mut StdoutLock) -> Result<(), std::io::Error> {
        let mut cursor_x = 0;
        let mut cursor_y = 0;

        queue!(stdout, terminal::Clear(ClearType::All))?;

        self.draw_text(stdout, &mut cursor_x, &mut cursor_y)?;

        let secs = self.timer.elapsed().as_secs();

        queue!(
            stdout,
            cursor::MoveTo(0, self.terminal_height - INFO_HEIGHT as u16)
        )?;

        print!(
            "Characters:\t{} / {}\r\nWords:\t\t{} / {}\r\nTime:\t\t{}:{:02}",
            self.typed.chars().count(),
            self.text.chars().count(),
            count_spaces(&self.typed),
            count_spaces(self.text) + 1,
            secs / 60,
            secs % 60,
        );

        queue!(stdout, cursor::MoveTo(cursor_x, cursor_y))?;

        stdout.flush()
    }

    pub fn draw_text(
        &mut self,
        stdout: &mut StdoutLock,
        cursor_x: &mut u16,
        cursor_y: &mut u16,
    ) -> Result<(), std::io::Error> {
        queue!(stdout, cursor::MoveTo(0, 0))?;

        let lines_to_skip = self.current_line.saturating_sub(1) as usize;
        let chars_to_skip = self.wrapped[..lines_to_skip]
            .iter()
            .map(|line| line.chars().count())
            .sum();

        let mut wrapped = self
            .wrapped
            .iter()
            .skip(lines_to_skip)
            .take(self.terminal_height as usize - INFO_HEIGHT);
        let mut typed = self.typed.chars().skip(chars_to_skip);
        let mut line_in_progress = EMPTY_STRING.chars();

        'outer: for (y, line) in wrapped.by_ref().enumerate() {
            line_in_progress = line.chars();

            for (x, c) in line_in_progress.by_ref().enumerate() {
                if let Some(typed_c) = typed.next() {
                    if typed_c == c {
                        queue!(
                            stdout,
                            SetForegroundColor(Color::Green),
                            SetBackgroundColor(Color::Reset)
                        )?;
                        print!("{}", c);
                    } else {
                        queue!(
                            stdout,
                            SetForegroundColor(Color::White),
                            SetBackgroundColor(Color::DarkRed)
                        )?;
                        print!("{}", typed_c);
                    }
                } else {
                    queue!(
                        stdout,
                        SetForegroundColor(Color::Grey),
                        SetBackgroundColor(Color::Reset)
                    )?;

                    print!("{}", c);

                    *cursor_x = x as u16;
                    *cursor_y = y as u16;

                    break 'outer;
                }
            }

            println!("\r");
        }

        for c in line_in_progress {
            print!("{}", c);
        }

        println!("\r");

        for line in wrapped {
            for c in line.chars() {
                print!("{}", c);
            }

            println!("\r");
        }

        Ok(())
    }

    pub fn run(&mut self) -> Result<(), std::io::Error> {
        let mut stdout = stdout().lock();

        loop {
            self.draw(&mut stdout)?;
            if event::poll(Duration::from_millis(500))? {
                match event::read()? {
                    Event::Key(key_event) => match key_event.code {
                        KeyCode::Char(c) => {
                            if key_event.modifiers.contains(KeyModifiers::CONTROL) {
                                if c == 'q' {
                                    break;
                                }
                            } else {
                                self.typed.push(c);
                                if self.typed.chars().count() == self.text.chars().count() {
                                    break;
                                }
                            }
                        }

                        KeyCode::Backspace => {
                            self.typed.pop();
                        }

                        _ => (),
                    },
                    Event::Resize(width, h) => {
                        self.wrapped = utils::word_wrap_keep_space(self.text, width as usize);
                        self.terminal_height = h;
                        queue!(stdout, terminal::Clear(ClearType::All))?;
                    }
                    _ => (),
                }
            }
            self.set_current_line();
        }
        Ok(())
    }

    fn set_current_line(&mut self) {
        let mut typed_count = self.typed.chars().count();
        for (i, count) in self
            .wrapped
            .iter()
            .map(|line| line.chars().count())
            .enumerate()
        {
            if let Some(num) = typed_count.checked_sub(count) {
                typed_count = num;
            } else {
                self.current_line = i;
                break;
            }
        }
    }

    pub fn typed_ref(&self) -> &str {
        &self.typed
    }

    pub fn timer_ref(&self) -> &Timer {
        &self.timer
    }
}
