use crate::base::*;

pub fn parse_sysctl(config: &Config) -> Result<Vec<Behavior>, String> {
    let mut writes = false;
    let mut result = vec![];
    let mut iterator = config.args.iter();
    loop {
        match iterator.next().map(|s| s.as_str()) {
            Some("--write") | Some("-w") => {
                writes = true;
            }
            Some(opt) => {
                if opt.starts_with("-") {
                    return Err(format!("permly: sysctl only works with -w/--write"));
                } else if let Some((k, v)) = opt.split_once('=') {
                    if config.now {
                        result.push(Behavior::Run {
                            cmd: vec![
                                "sysctl".to_string(),
                                "-w".to_string(),
                                format!("{}={}", k, v),
                            ],
                        })
                    }
                    result.push(Behavior::AppendLineToFile {
                        filename: "/etc/sysctl.d/99-permly.conf".to_string(),
                        line: format!("{} = {}", k, v),
                    })
                }
            }
            None => {
                break;
            }
        }
    }

    if writes {
        Ok(result)
    } else {
        Err(format!("permly: sysctl only works with -w/--write"))
    }
}
