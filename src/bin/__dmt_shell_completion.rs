use dmt::commands::{CommandHelper, CommandOptions};
use shell_completion::{BashCompletionInput, CompletionInput, CompletionSet};
use std::process::Command;

fn main() {
    let input = BashCompletionInput::from_env().unwrap();
    complete(input).suggest();
}

fn remove_suggestions(package: &str) -> Result<Vec<String>, String> {
    let command = Command::new("yay")
        .arg("-Qeq")
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8(command.stdout).unwrap();
    let stderr = String::from_utf8(command.stderr).unwrap();

    if !stderr.is_empty() {
        return Err(stderr);
    }

    let mut output: Vec<String> = stdout
        .lines()
        .filter(|line| line.contains(package))
        .map(|line| line.to_string())
        .collect();

    if output.len() > 0 {
        output.remove(0);
    }

    Ok(output)
}

fn add_suggestions(package: &str) -> Result<Vec<String>, String> {
    let command = Command::new("yay")
        .arg("-Pc")
        .output()
        .expect("failed to execute process");

    let stdout = String::from_utf8(command.stdout).unwrap();
    let stderr = String::from_utf8(command.stderr).unwrap();

    if !stderr.is_empty() {
        return Err(stderr);
    }

    let mut output: Vec<String> = stdout
        .lines()
        .filter(|line| line.contains(package))
        .map(|line| line.split_whitespace().next().unwrap().to_string())
        .collect();

    if output.len() > 0 {
        output.remove(0);
    }

    Ok(output)
}

fn complete(input: impl CompletionInput) -> Vec<String> {
    match input.arg_index() {
        0 => unreachable!(),
        1 => {
            vec![
                "-a  --add".to_string(),
                "-r  --remove".to_string(),
                "-u  --update".to_string(),
                "-h  --help".to_string(),
            ]
        }
        2 => {
            let command = CommandHelper::attribute_option(input.previous_word());

            match command {
                CommandOptions::Add => add_suggestions(input.current_word()).unwrap(),
                CommandOptions::Remove => remove_suggestions(input.current_word()).unwrap(),
                _ => vec![],
            }
        }
        _ => vec![],
    }
}
