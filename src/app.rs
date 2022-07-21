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

pub struct App<'a> {
    terminal_height: u16,
    text: &'a str,
    timer: Timer,
    typed: String,
    wrapped: Box<[&'a str]>,
}

impl<'a> App<'a> {
    pub fn draw(&self, stdout: &mut StdoutLock) -> Result<(), std::io::Error> {
        queue!(stdout, cursor::MoveTo(0, 0))?;
        let mut cursor_x = 0;
        let mut cursor_y = 0;

        self.draw_text(stdout, &mut cursor_x, &mut cursor_y)?;

        queue!(stdout, cursor::MoveTo(0, self.terminal_height - 3))?;

        let secs = self.timer.elapsed().as_secs();

        print!(
            "Characters:\t{} / {}\r\nWords:\t\t{} / {}\r\nTime:\t\t{}:{:02}",
            self.typed.chars().count(),
            self.text.chars().count(),
            count_spaces(&self.typed),
            count_spaces(self.text) + 1,
            secs / 60,
            secs % 60,
        );

        queue!(stdout, cursor::MoveTo(cursor_x as u16, cursor_y as u16))?;

        stdout.flush()
    }
    pub fn draw_text(
        &self,
        stdout: &mut StdoutLock,
        cursor_x: &mut usize,
        cursor_y: &mut usize,
    ) -> Result<(), std::io::Error> {
        static EMPTY_STRING: String = String::new();

        let mut wrapped = self.wrapped.iter();
        let mut typed = self.typed.chars();
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

                    *cursor_x = x;
                    *cursor_y = y;

                    break 'outer;
                }
            }

            new_line();
        }

        for c in line_in_progress {
            print!("{}", c);
        }

        new_line();

        for line in wrapped {
            for c in line.chars() {
                print!("{}", c);
            }

            new_line();
        }

        Ok(())
    }

    pub fn new(text: &'a str) -> Self {
        let (wrapped, terminal_height) = {
            let (width, height) = terminal::size().unwrap();
            (word_wrap_keep_space(text, width as usize), height)
        };
        let typed = String::with_capacity(text.len());
        let timer = Timer::new();
        Self {
            text,
            typed,
            wrapped,
            terminal_height,
            timer,
        }
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
                                if self.typed.len() == self.text.len() {
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
        }
        Ok(())
    }

    pub fn typed_ref(&self) -> &str {
        &self.typed
    }

    pub fn timer_ref(&self) -> &Timer {
        &self.timer
    }
}

impl<'a> Drop for App<'a> {
    fn drop(&mut self) {}
}

fn new_line() {
    println!("\r");
}
