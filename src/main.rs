mod app;
mod setup;
mod time;
mod utils;

use app::App;
use time::Timer;
use utils::count_spaces;

fn main() {
    let terminal_setup = setup::terminal();

    let text = setup::get_text();

    let mut app = App::new(&text);

    let result = app.run();

    terminal_setup.reset();

    if let Err(e) = result {
        eprintln!("{e}");
    } else {
        println!(
            "Words / min:\t{:.1}\nAccuracy:\t{:.1} %",
            calculate_wpm(&text, app.typed_ref(), app.timer_ref()),
            calculate_accuracy(&text, app.typed_ref()) * 100.0,
        );
    }
}

fn calculate_wpm(text: &str, typed: &str, timer: &Timer) -> f64 {
    let word_count = if typed.chars().count() == text.chars().count() {
        count_spaces(text) + 1
    } else {
        text.chars()
            .take(typed.chars().count())
            .filter(|c| *c == ' ')
            .count()
    };
    word_count as f64 / (timer.elapsed().as_secs_f64() / 60.0)
}

fn calculate_accuracy(text: &str, typed: &str) -> f64 {
    typed
        .chars()
        .zip(text.chars())
        .filter(|(c, correct)| c == correct)
        .count() as f64
        / typed.chars().count() as f64
}
