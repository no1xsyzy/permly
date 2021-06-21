use crate::base::*;

pub fn parse_export(config: &Config) -> Result<Vec<Behavior>, String> {
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
