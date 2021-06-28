use std::path::PathBuf;

use crate::base::*;

fn resolve_dir(dir: String) -> Result<String, String> {
    Ok(PathBuf::from(dir)
        .canonicalize()
        .map_err(|_| "no such dir")?
        .to_str()
        .ok_or("unable to transform it to string")?
        .to_string())
}

pub fn parse_mount(config: &Config) -> Result<Vec<Behavior>, String> {
    let mut options: Vec<String> = vec![];
    let mut device: Option<String> = None;
    let mut mountpoint: Option<String> = None;
    let mut fstype: Option<String> = None;
    let mut iterator = config.args.iter();
    loop {
        match iterator.next().map(|s| s.as_str()) {
            Some("--options") | Some("-o") => {
                if let Some(arg) = iterator.next() {
                    options.push(arg.clone());
                } else {
                    return Err(format!("mount: option requires an argument -- '--options'"));
                }
            }
            Some("--types") | Some("-t") => {
                if fstype.is_some() {
                    return Err(format!("mount: multiple --types"));
                } else if let Some(arg) = iterator.next() {
                    fstype = Some(arg.clone());
                } else {
                    return Err(format!("mount: option requires an argument -- '--types'"));
                }
            }
            Some("--bind") | Some("-B") => {
                if fstype.is_some() {
                    return Err(format!("mount: bad usage"));
                } else {
                    fstype = Some("none".to_string());
                    options.push("bind".to_string());
                }
            }
            Some("--readonly") | Some("-r") => {
                options.push("ro".to_string());
            }
            Some(opt) => {
                if opt.starts_with("-") {
                    return Err(format!("mount: unknown option -- '{}'", opt));
                } else if device.is_none() {
                    device = Some(opt.to_string());
                } else if mountpoint.is_none() {
                    mountpoint = Some(opt.to_string());
                } else {
                    return Err(format!("Too much arguments"));
                }
            }
            None => {
                break;
            }
        }
    }

    let device = resolve_dir(device.ok_or("No device")?)?;
    let mountpoint = resolve_dir(mountpoint.ok_or("No mountpoint")?)?;
    let fstype = fstype.ok_or("No fs_type")?;
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
