use crate::base::*;

pub fn parse_mount(config: &Config) -> Result<Vec<Behavior>, String> {
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
