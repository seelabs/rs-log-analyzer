use std::collections::HashMap;
use std::io::Write;

use crate::log_line::LogLine;

struct JobLatency {
    job: String,
    run: u64,  // run time ms
    wait: u64, // wait time ms
}

impl JobLatency {
    fn from_json_value(v: serde_json::Value) -> Option<Self> {
        // Typical json value
        // {
        //     "jlogId": 115,
        //     "job": "InboundLedger",
        //     "run(ms)": 0,
        //     "wait(ms)": 1160
        // }
        if let serde_json::Value::Object(m) = v {
            let job = m.get("job")?.as_str()?.to_string();
            let run = m.get("run(ms)")?.as_u64()?;
            let wait = m.get("wait(ms)")?.as_u64()?;
            return Some(JobLatency { job, run, wait });
        }
        return Option::None;
    }
}

struct JobLatencyCollection {
    run: Vec<u64>,
    wait: Vec<u64>,
}

impl JobLatencyCollection {
    // mutable self so the collections may be sorted
    fn write_stats(&mut self, job_name: &str, out_file: &mut std::fs::File) {
        assert!(self.run.len() == self.wait.len());

        if self.run.is_empty() {
            return;
        }

        let write_histogram = |out_file: &mut std::fs::File, v: &Vec<u64>| {
            // number of starts printed at the bin
            // with the max value
            let histogram_max_stars = 32;

            // histogram countains counts of values less than or equal to the bin
            // boundary. The boundary is (1 << (bin_index -1)), except for the 0th index,
            // which is for zeros.
            let mut h: [i32; 64] = [0; 64];
            let mut max_bin_index: u64 = 0;
            for d in v {
                let bin: usize = if *d == 0 {
                    0
                } else {
                    65 - (d - 1).leading_zeros()
                } as usize;
                h[bin] += 1;
                max_bin_index = max_bin_index.max(bin as u64);
            }

            let max_count = h.iter().max().unwrap();
            let mut any_written = false;
            for (index, count) in h.iter().enumerate() {
                if *count == 0 && !any_written {
                    continue;
                }
                let bin_index: u64 = if index == 0 { 0 } else { 1u64 << (index - 1) };
                write!(out_file, "{:>6} : {:<6} ", bin_index, count).unwrap();
                let num_stars = count * histogram_max_stars / max_count;
                for _ in 0..num_stars {
                    write!(out_file, "*").unwrap();
                }
                write!(out_file, "\n").unwrap();
                any_written = true;
                if (index as u64) == max_bin_index {
                    break;
                }
            }
        };

        self.run.sort();
        self.wait.sort();
        let run_max = self.run[self.run.len() - 1];
        let run_ave = (Iterator::sum::<u64>(self.run.iter()) as f64) / (self.run.len() as f64);
        let wait_max = self.wait[self.wait.len() - 1];
        let wait_ave = (Iterator::sum::<u64>(self.wait.iter()) as f64) / (self.wait.len() as f64);
        write!(
            out_file,
            "Job: {}: Max Run: {} Max Wait: {} Ave Run: {:.2} Ave Wait: {:.2}\n",
            job_name, run_max, wait_max, run_ave, wait_ave
        )
        .unwrap();
        write!(out_file, "Run histogram:\n").unwrap();
        write_histogram(out_file, &self.run);
        write!(out_file, "\n").unwrap();
        write!(out_file, "Wait histogram:\n").unwrap();
        write_histogram(out_file, &self.wait);
        write!(out_file, "\n\n").unwrap();
    }
}

// the `onlyDataAsJson` parameter controls if the whole log line will be written
// as json (better for computers) or just the json data (more readable for humans)
pub fn job_latency_stats(log_lines: &Vec<LogLine>, out_file_name: &std::path::PathBuf) {
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

    let mut stats = HashMap::<String, JobLatencyCollection>::new();

    for l in log_lines {
        if l.msg != "Job latency" {
            continue;
        }
        let jdata = l.data_to_json_value();
        if jdata.is_none() {
            errors.push(l);
            continue;
        }
        let latency = JobLatency::from_json_value(jdata.unwrap());
        if latency.is_none() {
            errors.push(l);
            continue;
        }
        let latency = latency.unwrap();

        if let Some(v) = stats.get_mut(&latency.job) {
            v.run.push(latency.run);
            v.wait.push(latency.wait);
        } else {
            stats.insert(
                latency.job,
                JobLatencyCollection {
                    run: Vec::with_capacity(32),
                    wait: Vec::with_capacity(32),
                },
            );
        }
    }

    for (k, v) in &mut stats {
        v.write_stats(&k, &mut out_file);
    }

    if !errors.is_empty() {
        eprintln!("Error: Invalid json data >>>> ");
        for e in &errors {
            eprintln!("{:?}", e);
        }
        eprintln!("End Invalid json data <<<< ");
    }
}
