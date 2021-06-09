use std::io::prelude::*;
use std::process::exit;

mod lib;
use crate::lib::*;

fn main() {
    let mut config = Config {
        dry_run: false,
        now: false,
        cmd: None,
        args: Vec::new(),
    };

    let mut done_self = false;
    let mut done_opts = false;

    for arg in std::env::args() {
        if !done_self {
            done_self = true;
        } else if config.cmd.is_some() {
            config.args.push(arg);
        } else if done_opts {
            config.cmd = Some(arg);
        } else if arg == "--dry-run" {
            config.dry_run = true;
        } else if arg == "--now" {
            config.now = true;
        } else if arg == "--" {
            done_opts = true;
        } else if arg.starts_with("--") {
            println!("Unknown opt {}", arg);
            exit(1);
        } else if arg.starts_with("-") {
            for c in arg.chars() {
                if c == 'n' {
                    config.dry_run = true;
                }
            }
        } else {
            config.cmd = Some(arg);
            done_opts = true;
        }
    }

    match parse(&config) {
        Ok(bs) => {
            for b in bs.into_iter() {
                match do_it(&b, &config) {
                    Ok(()) => {}
                    Err(e) => exit(e),
                }
            }
        }
        Err(s) => {
            println!("{}", s);
            exit(1)
        }
    }
}

fn do_it(behavior: &Behavior, config: &Config) -> Result<(), i32> {
    if config.dry_run {
        println!("sayperm: {}", behavior);
        Ok(())
    } else {
        behavior.exec()
    }
}

trait Execute {
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
