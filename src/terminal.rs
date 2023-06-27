use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::{cursor, event, execute, terminal, QueueableCommand};
use std::collections::VecDeque;
use std::io::{stdout, Stdout, Write};

use crate::exec::ExecContext;
use crate::parse;

pub fn setup() -> crossterm::Result<()> {
    // let (_width, height) = terminal_size().expect("failed to get terminal size");
    // println!(
    //     "{}{}",
    //     termion::clear::All,
    //     termion::cursor::Goto(1, height),
    // );
    execute!(stdout(), terminal::Clear(terminal::ClearType::Purge))?;
    terminal::enable_raw_mode()?;
    Ok(())
}

pub fn teardown() -> crossterm::Result<()> {
    terminal::disable_raw_mode()?;
    Ok(())
}

struct Prompt {
    left: VecDeque<char>,
    right: VecDeque<char>,
}
impl Prompt {
    fn new() -> Prompt {
        Prompt {
            left: VecDeque::new(),
            right: VecDeque::new(),
        }
    }
    fn add_char(&mut self, c: char) {
        self.left.push_back(c);
    }
    fn move_left_one(&mut self) {
        if let Some(c) = self.left.pop_back() {
            self.right.push_front(c);
        }
    }
    fn move_right_one(&mut self) {
        if let Some(c) = self.right.pop_front() {
            self.left.push_back(c);
        }
    }
    fn backspace_one(&mut self) {
        self.left.pop_back();
    }
    fn delete_one(&mut self) {
        self.right.pop_front();
    }
    fn render(&self, out: &mut Stdout) -> crossterm::Result<()> {
        // TODO multiline
        let (_width, height) = crossterm::terminal::size()?;
        out.queue(cursor::MoveTo(0, height))?
            .queue(terminal::Clear(terminal::ClearType::FromCursorDown))?;
        print!("> ");
        print!("{}", self.left.iter().collect::<String>());
        print!("{}", self.right.iter().collect::<String>());
        out.queue(cursor::MoveTo(
            (self.left.len() + 2).try_into().unwrap(),
            height,
        ))?;
        out.flush()?;
        Ok(())
    }
    fn build(&self) -> String {
        self.left.iter().chain(self.right.iter()).collect()
    }
}

pub fn event_loop(context: &mut ExecContext) -> crossterm::Result<()> {
    let mut out = stdout();
    // let mut history: Vec<String> = vec![];
    let mut prompt = Prompt::new();
    prompt.render(&mut out)?;
    loop {
        if let Event::Key(event) = event::read()? {
            // println!("{:?}", event);
            if event.modifiers.contains(KeyModifiers::CONTROL) {
                match event.code {
                    KeyCode::Char('c') => {
                        break;
                    }
                    _ => {}
                }
            } else {
                match event.code {
                    KeyCode::Char(c) => prompt.add_char(c),
                    KeyCode::Left => prompt.move_left_one(),
                    KeyCode::Right => prompt.move_right_one(),
                    KeyCode::Backspace => prompt.backspace_one(),
                    KeyCode::Delete => prompt.delete_one(),
                    KeyCode::Enter => {
                        terminal::disable_raw_mode()?;
                        println!();
                        if let Some(ast) = parse(&prompt.build()) {
                            context.execute(ast);
                        } else {
                            println!("Invalid syntax");
                        }
                        terminal::enable_raw_mode()?;
                        prompt = Prompt::new();
                    }
                    _ => {}
                }
                prompt.render(&mut out)?;
            }
        }
    }
    Ok(())
}
