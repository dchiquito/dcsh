use crossterm::event::{Event, KeyCode, KeyModifiers};
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
    execute!(
        stdout(),
        terminal::Clear(terminal::ClearType::All),
        terminal::Clear(terminal::ClearType::Purge)
    )?;
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
    fn from_history(source: &str) -> Prompt {
        Prompt {
            left: source.chars().collect(),
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

struct History {
    commands: Vec<String>,
    cursor: usize,
}
impl History {
    fn new() -> History {
        History {
            commands: vec![],
            cursor: 0,
        }
    }
    fn push(&mut self, command: String) {
        self.commands.push(command);
        self.cursor = self.commands.len();
    }
    fn up(&mut self) -> &str {
        if self.cursor > 0 {
            self.cursor -= 1;
        }
        &self.commands[self.cursor]
    }
    fn down(&mut self) -> &str {
        if self.cursor < self.commands.len() {
            self.cursor += 1;
        }
        if self.cursor == self.commands.len() {
            ""
        } else {
            &self.commands[self.cursor]
        }
    }
}

pub fn event_loop(context: &mut ExecContext) -> crossterm::Result<()> {
    let mut out = stdout();
    let mut history = History::new();
    let mut prompt = Prompt::new();
    prompt.render(&mut out)?;
    loop {
        if let Event::Key(event) = event::read()? {
            // println!("{:?}", event);
            if event.modifiers.contains(KeyModifiers::CONTROL) {
                match event.code {
                    KeyCode::Char('d') => {
                        break;
                    }
                    _ => {}
                }
            } else {
                match event.code {
                    KeyCode::Char(c) => prompt.add_char(c),
                    KeyCode::Left => prompt.move_left_one(),
                    KeyCode::Right => prompt.move_right_one(),
                    KeyCode::Up => prompt = Prompt::from_history(history.up()),
                    KeyCode::Down => prompt = Prompt::from_history(history.down()),
                    KeyCode::Backspace => prompt.backspace_one(),
                    KeyCode::Delete => prompt.delete_one(),
                    KeyCode::Enter => {
                        terminal::disable_raw_mode()?;
                        println!();
                        let command = prompt.build();
                        if let Some(ast) = parse(&command) {
                            context.execute(ast);
                        } else {
                            println!("Invalid syntax");
                        }
                        history.push(command);
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
