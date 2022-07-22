use std::{
    io::{stdout, StdoutLock, Write},
    mem::MaybeUninit,
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

const BOTTOM_TEXT_HEIGHT: usize = 3;

pub struct App<'a> {
    cursor_x: u16,
    cursor_y: u16,
    stdout: StdoutLock<'a>,
    terminal_height: u16,
    text: &'a str,
    timer: Timer,
    typed: String,
    wrapped: Box<[&'a str]>,
}

impl<'a> App<'a> {
    pub fn new(text: &'a str) -> Self {
        debug_assert!(!text.is_empty());
        let (wrapped, terminal_height) = {
            let (width, height) = terminal::size().unwrap();
            (word_wrap_keep_space(text, width as usize), height)
        };
        let typed = String::with_capacity(text.len());
        let timer = Timer::new();
        let stdout = stdout().lock();
        Self {
            cursor_x: 0,
            cursor_y: 0,
            stdout,
            terminal_height,
            text,
            timer,
            typed,
            wrapped,
        }
    }

    pub fn draw(&mut self) -> Result<(), std::io::Error> {
        self.draw_text()?;

        let secs = self.timer.elapsed().as_secs();

        queue!(
            self.stdout,
            cursor::MoveTo(0, self.terminal_height - BOTTOM_TEXT_HEIGHT as u16)
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

        queue!(self.stdout, cursor::MoveTo(self.cursor_x, self.cursor_y))?;

        self.stdout.flush()
    }

    pub fn draw_text(&mut self) -> Result<(), std::io::Error> {
        queue!(self.stdout, cursor::MoveTo(0, 0))?;

        let mut wrapped = self.wrapped.iter();
        let mut typed = self.typed.chars();
        let mut line_in_progress = MaybeUninit::uninit();

        'outer: for (y, line) in wrapped.by_ref().enumerate() {
            line_in_progress.write(line.chars());

            for (x, c) in unsafe { line_in_progress.assume_init_mut() }.enumerate() {
                if let Some(typed_c) = typed.next() {
                    if typed_c == c {
                        queue!(
                            self.stdout,
                            SetForegroundColor(Color::Green),
                            SetBackgroundColor(Color::Reset)
                        )?;
                        print!("{}", c);
                    } else {
                        queue!(
                            self.stdout,
                            SetForegroundColor(Color::White),
                            SetBackgroundColor(Color::DarkRed)
                        )?;
                        print!("{}", typed_c);
                    }
                } else {
                    queue!(
                        self.stdout,
                        SetForegroundColor(Color::Grey),
                        SetBackgroundColor(Color::Reset)
                    )?;

                    print!("{}", c);

                    self.cursor_x = x as u16;
                    self.cursor_y = y as u16;

                    break 'outer;
                }
            }

            println!("\r");
        }

        for c in unsafe { line_in_progress.assume_init() } {
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
        loop {
            self.draw()?;
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
                        queue!(self.stdout, terminal::Clear(ClearType::All))?;
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
