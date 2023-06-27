use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::collections::HashMap;

use crate::{command::SyntaxError, parse::Statement};

lazy_static! {
    static ref RE_SIMPLE_VARIABLE: Regex = Regex::new(r"\$([a-zA-Z0-9]+)").unwrap();
    static ref RE_BRACED_VARIABLE: Regex = Regex::new(r"\$\{[ \t]*([a-zA-Z0-9]+)[ \t]*\}").unwrap();
    static ref RE_SPACE_SEPERATOR: Regex = Regex::new(r"[ \t]+").unwrap();
}
#[derive(Debug)]
pub struct ExecContext {
    strings: HashMap<String, String>,
}

impl ExecContext {
    pub fn new() -> ExecContext {
        ExecContext {
            strings: HashMap::new(),
        }
    }
    pub fn execute(&mut self, statements: Vec<Statement>) {
        for statement in statements {
            match statement {
                Statement::Assignment(variable, expression) => {
                    self.exec_assignment(variable, expression)
                }
                Statement::Command(command) => self.exec_command(command),
                Statement::If(conditional, if_block, else_block) => {
                    self.exec_if(conditional, if_block, else_block)
                }
            }
        }
    }
    pub fn exec_assignment(&mut self, variable: String, expression: String) {
        self.strings
            .insert(variable, self.perform_substitution(&expression));
    }
    fn exec_command(&mut self, command: String) {
        match crate::command::exec_command(self, &command) {
            Err(SyntaxError::CommandNotFound(command)) => {
                eprintln!("dcsh: command not found: {}", command)
            }
            Err(_) => eprintln!("dcsh: unknown error"),
            Ok(_) => {}
        }
    }
    fn exec_if(
        &mut self,
        conditional: String,
        if_block: Vec<Statement>,
        else_block: Vec<Statement>,
    ) {
        println!("IFFY {} {:?} {:?}", conditional, if_block, else_block);
    }
    pub fn perform_substitution(&self, source: &str) -> String {
        let empty = "".to_string();
        let simple_vars = RE_SIMPLE_VARIABLE.replace_all(source, |caps: &Captures| {
            self.strings.get(&caps[1]).unwrap_or(&empty)
        });
        let braced_vars = RE_BRACED_VARIABLE.replace_all(&simple_vars, |caps: &Captures| {
            self.strings.get(&caps[1]).unwrap_or(&empty)
        });
        braced_vars.to_string()
    }
}
