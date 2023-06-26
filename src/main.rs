mod exec;
mod parse;

use exec::ExecContext;
use std::io::Write;

use crate::parse::parse;

fn main() {
    // let ast = parse("foo=bar\nbaz=$foo\nbingo=${ baz }\nif foo:\n  ls\n\n      \n\t\n\n  pwd\npwd")
    // .unwrap();
    // println!("{:?}", ast);
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
