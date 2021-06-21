use std::process::exit;

mod base;
mod export;
mod mount;
use crate::base::*;
use crate::export::parse_export;
use crate::mount::parse_mount;

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

fn parse(config: &Config) -> Result<Vec<Behavior>, String> {
    match config.cmd.as_ref().map(|s| s.as_str()) {
        Some("mount") => parse_mount(&config),
        Some("export") => parse_export(&config),
        Some(cmd) => Err(format!("Command not supported! {}", cmd)),
        None => Err("No command is specified!".to_string()),
    }
}
