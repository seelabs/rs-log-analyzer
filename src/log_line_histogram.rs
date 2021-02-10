use itertools::Itertools;

use std::collections::BTreeSet;
use std::cmp::Ordering;
use std::io::Write;

use crate::log_line::{LogLevel, LogLine};

#[derive(Debug, Eq, PartialEq)]
struct HistogramElement<'a> {
    line: LogLine<'a>,
    count: u32,
}

impl<'a> PartialOrd for HistogramElement<'a> {
    // Sort my level first, then by count, then by line
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.line.level.cmp(&other.line.level) {
            Ordering::Less => return Some(Ordering::Greater),
            Ordering::Greater => return Some(Ordering::Less),
            Ordering::Equal => (),
        }
        match self.count.cmp(&other.count) {
            Ordering::Less => return Some(Ordering::Greater),
            Ordering::Greater => return Some(Ordering::Less),
            Ordering::Equal => (),
        }
        Some(self.line.cmp(&other.line))
    }
}

impl<'a> Ord for HistogramElement<'a> {
    // Sort my level first, then by count, then by line
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(&other).unwrap()
    }
}

// Compute the number of starting words a and b have in common
fn n_prefix(a: &str, b: &str) -> u32 {
    // if wa and wb contain numbers, ignore them
    let is_numeric = |a: &str| -> bool {
        for c in a.chars() {
            if c.is_numeric() {
                return true;
            }
        }
        return false;
    };

    let mut result: u32 = 0;
    for (wa, wb) in a.split(' ').zip(b.split(' ')) {
        if is_numeric(wa) && is_numeric(wb) {
            continue;
        }
        if wa != wb {
            return result;
        }
        result += 1;
    }
    return result;
}

pub fn to_histogram(
    log_lines: &BTreeSet<LogLine>,
    histogram_out_file_name: &Option<std::path::PathBuf>,
    grouped_out_file_name: &Option<std::path::PathBuf>,
) {
    let to_file = |fname: &Option<std::path::PathBuf>| -> Option<std::fs::File> {
        if fname.is_none() {
            return Option::None;
        }
        let fname = fname.as_ref().unwrap();
        match std::fs::File::create(fname) {
            Ok(file) => Some(file),
            _ => {
                eprintln!("Could not create file {} in to_histogram", fname.display());
                std::process::exit(1);
            }
        }
    };

    let histogram_out_file: Option<std::fs::File> = to_file(histogram_out_file_name);
    let mut grouped_out_file: Option<std::fs::File> = to_file(grouped_out_file_name);

    if histogram_out_file_name.is_none() && grouped_out_file.is_none() {
        return;
    }

    let mut write_group = |group: &Vec<LogLine>| {
        if grouped_out_file.is_none() {
            return;
        }
        let mut errors = Vec::with_capacity(1024);

        let mut out = grouped_out_file.as_mut().unwrap();
        write!(out, ">>>> Group Size: {}\n", group.len()).unwrap();
        for l in group {
            if l.write_mixed_json(&mut out) {
                write!(out, "\n").unwrap();
            } else {
                errors.push(l);
            }
        }
        write!(out, "<<<<\n").unwrap();

        if !errors.is_empty() {
            eprintln!("Error: Invalid json data >>>> ");
            for e in &errors {
                eprintln!("{:?}", e);
            }
            eprintln!("End Invalid json data <<<< ");
        }
    };

    // Ignore lines whose first word contains non alphabetic characters
    // Allow ':' '[' ']' '<' '>'
    // Allow decimal numbers
    let ignore = |msg: &str| -> bool {
        let mut has_alpha = false;
        let mut has_num = false;
        for c in msg.chars() {
            if c.is_whitespace() {
                return has_alpha == has_num;
            }
            if !c.is_alphabetic() {
                if c == ':' || c == '[' || c == ']' || c == '<' || c == '>' {
                    continue;
                }
                if c.is_numeric() {
                    has_num = true;
                }
            } else {
                has_alpha = true;
            }
        }
        return has_alpha == has_num;
    };

    let mut histogram = BTreeSet::<HistogramElement>::new();
    let mut ignored = BTreeSet::<LogLine>::new();

    if log_lines.is_empty() {
        return;
    }

    let mut cur_group = Vec::<LogLine>::with_capacity(512);
    // insert the first line into the group
    for first in log_lines {
        // TODO: Use log_lines.first() when it stablizes
        cur_group.push(first.clone());
        break;
    }

    let is_new_group = |n: u32, prev_n_prefix: u32, prev: &LogLine, cur: &LogLine| -> bool {
        if prev.json_data.is_empty() != cur.json_data.is_empty() {
            return true;
        }

        if !prev.json_data.is_empty() && !cur.json_data.is_empty() {
            return prev.msg != cur.msg;
        }

        assert!(prev.json_data.is_empty() && cur.json_data.is_empty());

        return (!(prev_n_prefix == 0 && n != 0) && n < prev_n_prefix)
            || (prev_n_prefix == 0 && n == 0)
            || prev.level != cur.level
            || prev.module != cur.module;
    };

    let mut prev_n_prefix = 0;
    for (prev, cur) in log_lines.iter().tuple_windows() {
        let n = n_prefix(prev.msg, cur.msg);

        if cur.json_data.is_empty() && ignore(cur.msg) {
            ignored.insert(cur.clone());
            continue;
        }

        assert!(!cur_group.is_empty());
        if is_new_group(n, prev_n_prefix, prev, cur) {
            histogram.insert(HistogramElement {
                line: cur_group[0].clone(),
                count: cur_group.len() as u32,
            });
            write_group(&cur_group);
            cur_group.clear();
            cur_group.push(cur.clone());
        }

        prev_n_prefix = n;
        cur_group.push(cur.clone());
    }

    assert!(!cur_group.is_empty());
    histogram.insert(HistogramElement {
        line: cur_group[0].clone(),
        count: cur_group.len() as u32,
    });
    write_group(&cur_group);

    let mut prev_level = LogLevel::Trace;
    if let Some(mut out_file) = histogram_out_file {
        for HistogramElement { line: l, count: c } in &histogram {
            if l.level != prev_level {
                write!(out_file, "\n{:?}\n", l.level).unwrap();
                prev_level = l.level;
            }
            write!(out_file, "{} : ", c).unwrap();
            l.write_mixed_json(&mut out_file);
            write!(out_file, "\n\n").unwrap();
        }
    }

    if !ignored.is_empty() {
        eprintln!("\n\nIgnored Line In Histogram:");
    }
    for i in ignored {
        eprintln!("{}", i.line);
    }
}
