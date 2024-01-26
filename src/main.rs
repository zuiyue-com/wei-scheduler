#[macro_use]
extern crate wei_log;

use wei_single::SingleInstance;

use wei_job_scheduler::{JobScheduler, Job};
use std::time::Duration;

use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;

fn parse_line(line: &str) -> (String, String) {
    let parts: Vec<&str> = line.split_whitespace().collect();

    if parts.len() < 7 {
        info!("Invalid line: {}", line);
        return ("".to_string(), "".to_string());
    }

    let parts_one = parts[0..6].join(" ");
    let parts_two = parts[6..].join(" ");

    (parts_one, parts_two)
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    wei_windows::init();

    let instance = SingleInstance::new("wei-scheduler")?;
    if !instance.is_single() { 
        std::process::exit(1);
    };

    let path = Path::new("crontab.dat");
    let file = File::open(&path)?;
    let reader = io::BufReader::new(file);

    let mut sched = JobScheduler::new();

    for line in reader.lines() {
        let line = line?;

        if line.trim().is_empty() {
            continue;
        }

        let (cron_expr, command) = parse_line(&line);

        if cron_expr.is_empty() || command.is_empty() {
            continue;
        }

        let command = if cfg!(target_os = "windows") {
            format!("cmd.exe /c {}", command)
        } else {
            format!("sh -c {}", command)
        };

        sched.add(Job::new( match cron_expr.parse() {
            Ok(expr) => expr,
            Err(e) => {
                info!("Invalid cron expression: {}", e);
                continue;
            }
        }, move || {
            let command: Vec<&str> = command.split_whitespace().collect();
            info!("command: {:?}", command);
            match wei_run::command(command[0], command[1..].to_vec()) {
                Ok(output) => {
                    info!("output: {:?}", output);
                },
                Err(e) => {
                    info!("error: {}", e);
                }
            };
        }));
    }

    loop {
        sched.tick();
        std::thread::sleep(Duration::from_millis(500));
    }
}