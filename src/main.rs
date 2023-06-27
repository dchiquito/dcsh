mod command;
mod exec;
mod parse;
mod terminal;

use exec::ExecContext;
use std::io::Write;

use parse::parse;
use terminal::setup;

fn main() {
    setup();
    let mut context = ExecContext::new();
    let stdin = std::io::stdin();
    let mut stdout = std::io::stdout();
    loop {
        print!("> ");
        stdout.flush().expect("failed to write to stdout");
        let mut buffer = String::new();
        stdin
            .read_line(&mut buffer)
            .expect("failed to read from stdin");
        if let Some(ast) = parse(&buffer) {
            context.execute(ast);
        } else {
            println!("Invalid syntax");
        }
    }
}
