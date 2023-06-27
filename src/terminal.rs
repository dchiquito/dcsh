use termion::terminal_size;

pub fn setup() {
    let (_width, height) = terminal_size().expect("failed to get terminal size");
    println!(
        "{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, height),
    );
}
