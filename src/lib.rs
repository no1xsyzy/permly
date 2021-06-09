use std::fmt;

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

pub fn parse(config: &Config) -> Result<Vec<Behavior>, String> {
    match config.cmd.as_ref().map(|s| s.as_str()) {
        Some("mount") => parse_mount(&config),
        Some("export") => parse_export(&config),
        Some(cmd) => Err(format!("Command not supported! {}", cmd)),
        None => Err("No command is specified!".to_string()),
    }
}

fn parse_mount(config: &Config) -> Result<Vec<Behavior>, String> {
    let mut options: Vec<String> = vec![];
    let mut device: Option<String> = None;
    let mut mountpoint: Option<String> = None;
    let mut fstype: Option<String> = None;
    let mut last_opt: Option<String> = None;
    for arg in config.args.iter() {
        match (arg.starts_with("-"), last_opt.as_ref().map(|s| s.as_str())) {
            (true, Some(opt)) => {
                return Err(format!("mount: option requires an argument -- '{}'", opt))
            }
            (true, None) => {
                last_opt = Some(arg.clone());
            }
            (false, Some("-o")) | (false, Some("--options")) => {
                options.push(arg.clone());
                last_opt = None;
            }
            (false, Some("-t")) | (false, Some("--types")) => {
                fstype = Some(arg.clone());
                last_opt = None;
            }
            (false, Some(opt)) => {
                println!("Unsupported opts -- '{}'", opt)
            }
            (false, None) => {
                if arg.starts_with("-") {
                    return Err(format!("Unsupported opts for `mount`: `{}`", arg));
                } else if device.is_none() {
                    device = Some(arg.clone());
                } else if mountpoint.is_none() {
                    mountpoint = Some(arg.clone());
                } else {
                    return Err(format!("Too much arguments"));
                }
            }
        }
    }
    if last_opt.is_some() {
        return Err(format!(
            "mount: option requires an argument -- '{}'",
            last_opt.unwrap()
        ));
    }

    let device = device.expect("No device");
    let mountpoint = mountpoint.expect("No mountpoint");
    let fstype = fstype.expect("No fs_type");
    let options = options.join(",");

    if config.now {
        let mut cmd = vec![
            "mount".to_string(),
            device.clone(),
            mountpoint.clone(),
            "-t".to_string(),
            fstype.clone(),
        ];
        if options != "" {
            cmd.push("-o".to_string());
            cmd.push(options.clone());
        }

        Ok(vec![
            Behavior::Run { cmd: cmd },
            Behavior::AppendLineToFile {
                filename: "/etc/fstab".to_string(),
                line: vec![device, mountpoint, fstype, options].join("\t"),
            },
        ])
    } else {
        Ok(vec![Behavior::AppendLineToFile {
            filename: "/etc/fstab".to_string(),
            line: vec![device, mountpoint, fstype, options].join("\t"),
        }])
    }
}

fn parse_export(config: &Config) -> Result<Vec<Behavior>, String> {
    if config.now {
        return Err("export doesn't support --now".to_string());
    }

    let mut result = vec![];
    for entry in config.args.iter() {
        if let Some((k, v)) = entry.split_once('=') {
            result.push(Behavior::AppendLineToFile {
                filename: "~/.profile".to_string(),
                line: format!("export {}='{}'", k, v),
            })
        } else if let Ok(val) = std::env::var(entry) {
            result.push(Behavior::AppendLineToFile {
                filename: "~/.profile".to_string(),
                line: format!("export {}='{}'", entry, val),
            })
        } else {
            result.push(Behavior::NoOperationReason {
                reason: format!("variable `{}` is not found in current environment", entry),
            })
        }
    }
    Ok(result)
}
