use std::fmt;
use std::io::prelude::*;

pub struct Config {
    pub dry_run: bool,
    pub now: bool,
    pub cmd: Option<String>,
    pub args: Vec<String>,
}

pub enum Behavior {
    Run { cmd: Vec<String> },
    AppendLineToFile { filename: String, line: String },
    NoOperationReason { reason: String },
}

pub trait Execute {
    fn exec(self: &Self) -> Result<(), i32>;
}

impl Execute for Behavior {
    fn exec(&self) -> Result<(), i32> {
        match self {
            Behavior::Run { cmd } => {
                let exitstatus = std::process::Command::new(&cmd[0])
                    .args(&cmd[1..])
                    .status()
                    .expect("No such runsys");
                match exitstatus.code() {
                    Some(0) => Ok(()),
                    Some(x) => Err(x),
                    None => Err(130),
                }
            }
            Behavior::AppendLineToFile { filename, line } => {
                let mut fp = match std::fs::OpenOptions::new().append(true).open(&filename) {
                    Ok(fp) => fp,
                    Err(_) => {
                        return Err(128);
                    }
                };
                match fp.write_all(line.as_bytes()) {
                    Ok(_) => Ok(()),
                    Err(_) => Err(129),
                }
            }
            Behavior::NoOperationReason { reason: _ } => Ok(()),
        }
    }
}

impl fmt::Display for Behavior {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Behavior::Run { cmd } => {
                write!(f, "will run: {}", format_cmd_as_sh(cmd))
            }
            Behavior::AppendLineToFile { filename, line } => {
                write!(f, "append to `{}`: {}", filename, line)
            }
            Behavior::NoOperationReason { reason } => {
                write!(f, "skipping: {}", reason)
            }
        }
    }
}

fn format_cmd_as_sh(cmd: &Vec<String>) -> String {
    let mut j: Vec<String> = Vec::new();
    for c in cmd {
        if c == "" {
            j.push("''".to_string());
        } else if c.contains("'")
            || c.contains("\"")
            || c.contains("*")
            || c.contains("\\")
            || c.contains("?")
        {
            j.push(format!(
                "'{}'",
                c.replace("\\", "\\\\").replace("'", "'\\''")
            ))
        } else {
            j.push(c.clone());
        }
    }
    return j.join(" ");
}
