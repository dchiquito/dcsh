use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    static ref RE_ASSIGNMENT: Regex = Regex::new("\\A([a-zA-Z0-9]+)[ \t]*=[ \t]*([^\n]*)\n").unwrap();
    static ref RE_COMMAND: Regex = Regex::new("\\A([^\n]+)\n").unwrap();
    static ref RE_IF: Regex = Regex::new("\\Aif[ \t]+([^:]+):[ \t]*\n").unwrap();
    static ref RE_ELIF: Regex = Regex::new("\\Aelif[ \t]+([^:]+):[ \t]*\n").unwrap();
    static ref RE_ELSE: Regex = Regex::new("\\Aelse[ \t]*:[ \t]*\n").unwrap();
    static ref RE_INDENTATION: Regex = Regex::new("\\A[ \t]*").unwrap();
    static ref RE_EMPTY_LINES: Regex = Regex::new("\\A([ \t]*\n)*").unwrap();
    // static ref RE_IF: Regex = Regex::new("if ([^:]+):\n").unwrap();
}

#[derive(Debug)]
pub enum Statement {
    Assignment(String, String),
    Command(String),
    If(String, Vec<Statement>, Vec<Statement>),
}

pub fn parse(source: &str) -> Option<Vec<Statement>> {
    let source = &format!("{}\n", source);
    // TODO parse out \ line continuations at this point
    parse_code_block(source, "").map(|(statements, _source)| statements)
}

fn parse_code_block<'a>(
    mut source: &'a str,
    indentation: &'a str,
) -> Option<(Vec<Statement>, &'a str)> {
    let mut statements = vec![];
    while !source.is_empty() {
        let actual_indentation = RE_INDENTATION.find(source).unwrap().as_str();
        if actual_indentation != indentation {
            return Some((statements, source));
        }
        source = &source[indentation.len()..source.len()];
        if let Some((statement, new_source)) = parse_statement(source, indentation) {
            statements.push(statement);
            source = new_source;
        } else {
            return None;
        }
        let empty_lines = RE_EMPTY_LINES.find(source).unwrap().as_str();
        source = &source[empty_lines.len()..source.len()];
    }
    Some((statements, source))
}

fn parse_statement<'a>(source: &'a str, indentation: &'a str) -> Option<(Statement, &'a str)> {
    if let Some((statement, remainder)) = parse_if(source, indentation) {
        Some((statement, remainder))
    } else if let Some((statement, remainder)) = parse_assignment(source) {
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

fn parse_if<'a>(source: &'a str, indentation: &'a str) -> Option<(Statement, &'a str)> {
    if let Some(captures) = RE_IF.captures(source) {
        let condition = captures.get(1).unwrap().as_str().to_string();
        let remainder = &source[captures.get(0).unwrap().len()..source.len()];
        let new_indentation = find_indentation(remainder);
        // TODO ensure new_indentation is longer than old
        let (if_code, remainder) = parse_code_block(remainder, new_indentation).unwrap();
        if if_code.is_empty() {
            return None;
        }
        if let Some(captures) = RE_ELIF.captures(remainder) {
            // TODO elif
        } else if let Some(captures) = RE_ELSE.captures(remainder) {
            // TODO else
        }
        let else_code = vec![];
        let if_statement = Statement::If(condition, if_code, else_code);
        Some((if_statement, remainder))
    } else {
        None
    }
}

fn find_indentation(source: &str) -> &str {
    let captures = RE_INDENTATION.captures(source).unwrap();
    captures.get(0).unwrap().as_str()
}
