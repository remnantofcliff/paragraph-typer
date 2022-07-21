use std::io::stdout;

use crossterm::{
    execute, queue,
    terminal::{self, enable_raw_mode},
};
use deunicode::deunicode;
use rand::prelude::SliceRandom;

const PARAGRAPHS_URL: &str = "https://contenttool.io/getRandomParagraph";

#[derive(serde::Deserialize)]
struct Paragraph {
    paragraph: Box<str>,
}

pub struct TerminalControl;

impl TerminalControl {
    pub fn start() -> TerminalControl {
        enable_raw_mode().expect("Failed to enable raw mode");

        queue!(
            stdout(),
            terminal::EnterAlternateScreen,
            terminal::DisableLineWrap,
        )
        .expect("Failed to do startup terminal actions");

        TerminalControl
    }
    pub fn stop(self) {
        let mut stdout = std::io::stdout().lock();
        execute!(stdout, terminal::LeaveAlternateScreen).unwrap();
    }
}

pub fn get_text() -> Box<str> {
    deunicode(
        &ureq::get(PARAGRAPHS_URL)
            .call()
            .expect("Could not get paragraphs from server")
            .into_json::<Box<[Paragraph]>>()
            .expect("Could not parse paragraphs")
            .choose(&mut rand::thread_rng())
            .expect("Server returned no paragraphs")
            .paragraph,
    )
    .into_boxed_str()
}
