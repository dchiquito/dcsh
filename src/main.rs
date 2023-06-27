mod command;
mod exec;
mod parse;
mod terminal;

use exec::ExecContext;

use parse::parse;

fn main() -> crossterm::Result<()> {
    terminal::setup()?;
    let mut context = ExecContext::new();
    terminal::event_loop(&mut context)?;
    terminal::teardown()?;
    Ok(())
}
