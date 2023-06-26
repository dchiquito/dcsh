mod exec;
mod parse;

use exec::ExecContext;

use crate::parse::parse;
fn main() {
    let ast =
        parse("foo=bar\nbaz=$foo\nbingo=${ baz }\nif foo:\n  ls\n\n      \n\t\n\n  pwd\ncd ~")
            .unwrap();
    println!("{:?}", ast);
    let mut context = ExecContext::new();
    context.execute(ast);
    println!("{:?}", context);
}
