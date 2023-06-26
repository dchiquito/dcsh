use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_ASSIGNMENT: Regex = Regex::new("([a-zA-Z0-9]+)[ \t]*=[ \t]*([^\n]*)\n").unwrap();
    static ref RE_COMMAND: Regex = Regex::new("([^\n]+)\n").unwrap();
}

#[derive(Debug)]
enum Statement {
    Assignment(String, String),
    Command(String),
}

fn parse_statement(source: &str) -> Option<(Statement, &str)> {
    if let Some((statement, remainder)) = parse_assignment(source) {
        Some((statement, remainder))
    } else if let Some((statement, remainder)) = parse_command(source) {
        Some((statement, remainder))
    } else {
        None
    }
}

fn parse_assignment(source: &str) -> Option<(Statement, &str)> {
    if let Some(captures) = RE_ASSIGNMENT.captures(source) {
        let variable = captures.get(1).unwrap().as_str().to_string();
        let expression = captures.get(2).unwrap().as_str().to_string();
        let assignment = Statement::Assignment(variable, expression);
        let remainder = &source[captures.get(0).unwrap().len()..source.len()];
        Some((assignment, remainder))
    } else {
        None
    }
}

fn parse_command(source: &str) -> Option<(Statement, &str)> {
    if let Some(captures) = RE_COMMAND.captures(source) {
        let expression = captures.get(1).unwrap().as_str().to_string();
        let command = Statement::Command(expression);
        let remainder = &source[captures.get(0).unwrap().len()..source.len()];
        Some((command, remainder))
    } else {
        None
    }
}

fn main() {
    println!("{:?}", parse_statement("foo = bar\n  baz=baz\n"));
}
