use logos::{Lexer, Logos};
use std::{
    fs::File,
    process::{ChildStdout, Command, Stdio},
};

use crate::exec::ExecContext;

#[derive(Logos, Debug, PartialEq, Eq)]
#[logos(skip r"[ \t]*")]
enum CommandToken {
    #[token("&&")]
    And,
    #[token("||")]
    Or,
    #[token(";")]
    Semicolon,
    #[token("|")]
    Pipe,
    #[token("<")]
    InputRedirect,
    #[token(">")]
    OutputRedirect,
    #[token("2>")]
    StderrRedirect,
    #[regex(r#""([^"]|\\")*""#)]
    String,
    #[regex(r"[^ \t;]+")]
    Word,
}

#[derive(Debug, PartialEq, Eq)]
struct Invocation {
    executable: String,
    args: Vec<String>,
    input_file: Option<String>,
    output_file: Option<String>,
    stderr_file: Option<String>,
}

impl Invocation {
    fn new(executable: &str) -> Invocation {
        Invocation {
            executable: executable.to_string(),
            args: vec![],
            input_file: None,
            output_file: None,
            stderr_file: None,
        }
    }
    #[allow(dead_code)]
    fn arg(mut self, arg: &str) -> Invocation {
        self.args.push(arg.to_string());
        self
    }
    #[allow(dead_code)]
    fn input_file(mut self, input_file: &str) -> Invocation {
        self.input_file = Some(input_file.to_string());
        self
    }
    #[allow(dead_code)]
    fn output_file(mut self, output_file: &str) -> Invocation {
        self.output_file = Some(output_file.to_string());
        self
    }
    #[allow(dead_code)]
    fn stderr_file(mut self, stderr_file: &str) -> Invocation {
        self.stderr_file = Some(stderr_file.to_string());
        self
    }
    fn command(&self) -> Command {
        let mut command = Command::new(&self.executable);
        command.args(&self.args);
        if let Some(input_file) = &self.input_file {
            let file = File::open(input_file).expect("failed to open input file");
            command.stdin(file);
        }
        if let Some(output_file) = &self.output_file {
            let file = File::create(output_file).expect("failed to open output file");
            command.stdout(file);
        }
        if let Some(stderr_file) = &self.stderr_file {
            let file = File::create(stderr_file).expect("failed to open stderr file");
            command.stderr(file);
        }
        command
    }
}

#[derive(Debug, PartialEq, Eq)]
enum InvocationChain {
    And,
    Or,
    Semicolon,
    Pipe,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SyntaxError {
    ExpectedString,
    InvalidSyntax,
}

fn parse_string(
    context: &ExecContext,
    lexer: &mut Lexer<CommandToken>,
) -> Result<String, SyntaxError> {
    let token = lexer.next();
    if let Some(Ok(CommandToken::Word)) = token {
        Ok(context.perform_substitution(lexer.slice()))
    } else if let Some(Ok(CommandToken::String)) = token {
        let slice = lexer.slice();
        Ok(context.perform_substitution(&slice[1..slice.len() - 1]))
    } else {
        Err(SyntaxError::ExpectedString)
    }
}

fn parse_single_invocation(
    context: &ExecContext,
    lexer: &mut Lexer<CommandToken>,
) -> Result<(Invocation, Option<InvocationChain>), SyntaxError> {
    let executable = parse_string(context, lexer)?;
    let mut invocation = Invocation::new(&executable);
    let mut token = lexer.next();
    loop {
        match token {
            Some(Ok(CommandToken::Word)) => invocation
                .args
                .push(context.perform_substitution(lexer.slice())),
            Some(Ok(CommandToken::String)) => {
                let slice = lexer.slice();
                invocation
                    .args
                    .push(context.perform_substitution(&slice[1..slice.len() - 1]))
            }
            Some(Ok(CommandToken::InputRedirect)) => {
                invocation.input_file = Some(parse_string(context, lexer)?)
            }
            Some(Ok(CommandToken::OutputRedirect)) => {
                invocation.output_file = Some(parse_string(context, lexer)?)
            }
            Some(Ok(CommandToken::StderrRedirect)) => {
                invocation.stderr_file = Some(parse_string(context, lexer)?)
            }
            Some(Ok(CommandToken::And)) => return Ok((invocation, Some(InvocationChain::And))),
            Some(Ok(CommandToken::Or)) => return Ok((invocation, Some(InvocationChain::Or))),
            Some(Ok(CommandToken::Semicolon)) => {
                return Ok((invocation, Some(InvocationChain::Semicolon)))
            }
            Some(Ok(CommandToken::Pipe)) => return Ok((invocation, Some(InvocationChain::Pipe))),
            Some(Err(_)) => return Err(SyntaxError::InvalidSyntax),
            None => return Ok((invocation, None)),
        }
        token = lexer.next();
    }
}

fn parse_command(
    context: &ExecContext,
    source: &str,
) -> Result<Vec<(Invocation, Option<InvocationChain>)>, SyntaxError> {
    let mut lexer = CommandToken::lexer(source);
    let mut invocations = vec![];
    loop {
        let (invocation, chain) = parse_single_invocation(context, &mut lexer)?;
        if chain.is_none() {
            invocations.push((invocation, chain));
            break;
        }
        invocations.push((invocation, chain));
    }
    Ok(invocations)
}

pub fn exec_command(context: &ExecContext, command: &str) -> Result<i32, SyntaxError> {
    let invocations = parse_command(context, command)?;
    let mut previous_stdout: Option<ChildStdout> = None;
    for (invocation, chain) in invocations {
        let mut command = invocation.command();
        if let Some(stdout) = previous_stdout {
            command.stdin(Stdio::from(stdout));
        }
        previous_stdout = None;
        // TODO join stdout and stderr when piping
        // TODO join < and piped stdin
        match chain {
            Some(InvocationChain::And) => {
                let status = command.status().expect("failed to spawn process");
                if !status.success() {
                    return Ok(status.code().unwrap());
                }
            }
            Some(InvocationChain::Or) => {
                let status = command.status().expect("failed to spawn process");
                if status.success() {
                    return Ok(status.code().unwrap());
                }
            }
            Some(InvocationChain::Pipe) => {
                command.stdout(Stdio::piped());
                let child = command.spawn().expect("failed to spawn process");
                previous_stdout = child.stdout;
            }
            Some(InvocationChain::Semicolon) => {
                command.status().expect("failed to spawn process");
            }
            None => {
                return Ok(command
                    .status()
                    .expect("failed to spawn process")
                    .code()
                    .unwrap());
            }
        }
    }
    Ok(0)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_command() {
        let mut context = ExecContext::new();
        assert_eq!(
            parse_command(&context, "ls"),
            Ok(vec![(Invocation::new("ls"), None)])
        );
        assert_eq!(
            parse_command(&context, "ls -al"),
            Ok(vec![(Invocation::new("ls").arg("-al"), None)])
        );
        assert_eq!(
            parse_command(&context, "ls -al | grep foo"),
            Ok(vec![
                (
                    Invocation::new("ls").arg("-al"),
                    Some(InvocationChain::Pipe)
                ),
                (Invocation::new("grep").arg("foo"), None)
            ])
        );
        assert_eq!(
            parse_command(&context, "ls -al | grep foo || touch foo"),
            Ok(vec![
                (
                    Invocation::new("ls").arg("-al"),
                    Some(InvocationChain::Pipe)
                ),
                (
                    Invocation::new("grep").arg("foo"),
                    Some(InvocationChain::Or)
                ),
                (Invocation::new("touch").arg("foo"), None)
            ])
        );
        assert_eq!(
            parse_command(&context, "ls -al && pwd"),
            Ok(vec![
                (Invocation::new("ls").arg("-al"), Some(InvocationChain::And)),
                (Invocation::new("pwd"), None)
            ])
        );
        assert_eq!(
            parse_command(&context, "ls -al; pwd"),
            Ok(vec![
                (
                    Invocation::new("ls").arg("-al"),
                    Some(InvocationChain::Semicolon)
                ),
                (Invocation::new("pwd"), None)
            ])
        );
        assert_eq!(
            parse_command(&context, "cat < foo"),
            Ok(vec![(Invocation::new("cat").input_file("foo"), None)])
        );
        assert_eq!(
            parse_command(&context, "cat > foo"),
            Ok(vec![(Invocation::new("cat").output_file("foo"), None)])
        );
        assert_eq!(
            parse_command(&context, "cat 2> foo"),
            Ok(vec![(Invocation::new("cat").stderr_file("foo"), None)])
        );
        context.exec_assignment("eecchhoo".to_string(), "echo".to_string());
        assert_eq!(
            parse_command(&context, "$eecchhoo foo"),
            Ok(vec![(Invocation::new("echo").arg("foo"), None)])
        );
        assert_eq!(
            parse_command(&context, "${eecchhoo} foo"),
            Ok(vec![(Invocation::new("echo").arg("foo"), None)])
        );
    }
}
