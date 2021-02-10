use crate::log_line::LogLine;
use std::io::Write;

// the `onlyDataAsJson` parameter controls if the whole log line will be written
// as json (better for computers) or just the json data (more readable for humans)
pub fn to_json(log_lines: &Vec<LogLine>, out_file_name: &std::path::PathBuf, only_data_as_json: bool) {
    let mut out_file = match std::fs::File::create(out_file_name) {
        Ok(file) => file,
        _ => {
            eprintln!(
                "Could not create file {} in to_json",
                out_file_name.display()
            );
            std::process::exit(1);
        }
    };

    let mut errors = Vec::with_capacity(1024);

    if !only_data_as_json {
        for l in log_lines {
            if let Some(v) = l.to_json_value() {
                write!(out_file, "{}\n", serde_json::to_string_pretty(&v).unwrap()).unwrap();
            } else {
                errors.push(l);
            }
        }
    } else {
        // Write mixed
        for l in log_lines {
            if l.write_mixed_json(&mut out_file) {
                write!(out_file, "\n").unwrap();
            } else {
                errors.push(l);
            }
        }
    }

    if !errors.is_empty() {
        eprintln!("Error: Invalid json data >>>> ");
        for e in &errors {
            eprintln!("{:?}", e);
        }
        eprintln!("End Invalid json data <<<< ");
    }
}
