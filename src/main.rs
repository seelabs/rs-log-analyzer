use std::collections::BTreeSet;
use std::error::Error;

use structopt::StructOpt;

mod job_latency;
mod log_line;
mod log_line_histogram;
mod memmap_log;
mod to_json;

use log_line::LogLine;

#[derive(StructOpt)]
struct Cli {
    #[structopt(
        short = "i",
        long = "input",
        help = "rippled generated input log file",
        parse(from_os_str)
    )]
    input_log_file: std::path::PathBuf,
    #[structopt(
        short = "h",
        long = "histogram",
        help = "histogram of log file lines",
        parse(from_os_str)
    )]
    histogram_file: Option<std::path::PathBuf>,
    #[structopt(
        short = "j",
        long = "json",
        help = "convert log file to json",
        parse(from_os_str)
    )]
    json_file: Option<std::path::PathBuf>,
    #[structopt(
        short = "g",
        long = "grouped",
        help = "grouped file of log file lines",
        parse(from_os_str)
    )]
    grouped_file: Option<std::path::PathBuf>,

    #[structopt(
        short = "l",
        long = "job_latency",
        help = "job latency stats",
        parse(from_os_str)
    )]
    job_latency_file: Option<std::path::PathBuf>,

    #[structopt(
        short = "m",
        long = "mixed-json",
        help = "Use mixed json when writing the json file"
    )]
    mixed_json: bool,
}

fn main() {
    let args = Cli::from_args();

    if args.histogram_file.is_none() && args.json_file.is_none() {
        eprintln!("Must specify at least one output file");
        std::process::exit(1);
    }

    let file = match memmap_log::MemmapLog::new(&args.input_log_file) {
        Err(why) => panic!(
            "Couldn't open {}: {}",
            args.input_log_file.display(),
            Error::to_string(&why)
        ),
        Ok(file) => file,
    };

    let mut lines_vec = Vec::<LogLine>::with_capacity(1024 * 1024);

    for l in file.as_str().lines() {
        if let Some(log_line) = LogLine::new(&l) {
            lines_vec.push(log_line)
        }
    }

    if let Some(out) = args.json_file {
        to_json::to_json(&lines_vec, &out, args.mixed_json);
    }

    if let Some(out) = args.job_latency_file {
        job_latency::job_latency_stats(&lines_vec, &out);
    }

    if args.histogram_file.is_some() || args.grouped_file.is_some() {
        let lines_set: BTreeSet<LogLine> = lines_vec.into_iter().collect();
        log_line_histogram::to_histogram(&lines_set, &args.histogram_file, &args.grouped_file);
    }
}
