use std::io::stdout;

use crossterm::{
    execute, queue,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use rand::prelude::SliceRandom;

const PARAGRAPHS_URL: &str = "https://contenttool.io/getRandomParagraph";

#[derive(serde::Deserialize)]
struct Paragraph {
    paragraph: Box<str>,
}

pub struct TerminalControl;

pub fn terminal_setup() -> TerminalControl {
    enable_raw_mode().expect("Failed to enable raw mode");

    queue!(
        stdout(),
        terminal::EnterAlternateScreen,
        terminal::DisableLineWrap,
    )
    .expect("Failed to do startup terminal actions");
    TerminalControl
}

impl TerminalControl {
    pub fn reset(self) {
        execute!(stdout(), terminal::LeaveAlternateScreen)
            .expect("Failed to do closing terminal actions");
        disable_raw_mode().expect("Failed to disable raw mode");
    }
}

pub fn get_text() -> Box<str> {
    ureq::get(PARAGRAPHS_URL)
        .call()
        .expect("Could not get paragraphs from server")
        .into_json::<Box<[Paragraph]>>()
        .expect("Could not parse paragraphs")
        .choose(&mut rand::thread_rng())
        .expect("Server returned no paragraphs")
        .paragraph
        .to_owned()
}
