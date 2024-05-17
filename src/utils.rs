use clap::ValueEnum;
use serde::{Deserialize, Serialize};
use std::{
    fs::OpenOptions,
    io::{Read, Write},
    path::PathBuf,
    process::Command,
};

use crate::error::LustreCollectorError;

#[derive(Serialize, Deserialize)]
struct CommandOutput {
    command: String,
    args: Vec<String>,
    stdout: String,
    stderr: String,
}

pub struct CommandMock {
    pub name: String,
    pub mode: CommandMode,
    pub path: Option<PathBuf>,
}

#[derive(ValueEnum, PartialEq, Debug, Clone, Copy)]
pub enum CommandMode {
    None,
    Record,
    Play,
}
impl CommandMock {
    pub fn with_mode(mut self, mode: CommandMode) -> Self {
        self.mode = mode;
        self
    }

    pub fn with_path(mut self, path: impl Into<PathBuf>) -> Self {
        self.path = Some(path.into());
        self
    }

    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            mode: CommandMode::None,
            path: None,
        }
    }
}

fn execute_command(command: &str, args: &[String]) -> std::io::Result<CommandOutput> {
    let output = Command::new(command).args(args).output()?;

    Ok(CommandOutput {
        command: command.to_string(),
        args: args.to_vec(),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

fn write_to_file(cmd_output: &CommandOutput, pb: PathBuf) -> std::io::Result<()> {
    let buffer = serde_json::to_string(cmd_output)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(pb)?;
    file.write_all(buffer.as_bytes())?;
    Ok(())
}

fn read_from_file(pb: PathBuf) -> std::io::Result<CommandOutput> {
    let mut file = OpenOptions::new().read(true).open(pb)?;

    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;
    let cmd_output: CommandOutput = serde_json::from_str(&buffer)?;
    Ok(cmd_output)
}

pub fn get_output(
    command: &str,
    args: Vec<String>,
    mock: CommandMock,
) -> Result<Vec<u8>, LustreCollectorError> {
    let CommandMock { name, mode, path } = mock;
    let pb = path.unwrap_or(PathBuf::from(""));
    let pb = pb.join(format!("{name}.json"));
    match mode {
        CommandMode::Record => {
            let cmd_output = execute_command(command, &args)?;
            write_to_file(&cmd_output, pb)?;
            Ok(cmd_output.stdout.into_bytes())
        }
        CommandMode::Play => {
            let cmd_output = read_from_file(pb)?;
            Ok(cmd_output.stdout.into_bytes())
        }
        CommandMode::None => {
            let cmd_output = execute_command(command, &args)?;
            Ok(cmd_output.stdout.into_bytes())
        }
    }
}
