use clap::{Parser, Subcommand};
use serde::{Deserialize, Serialize};
use chrono::{Utc, DateTime};
use std::fs::OpenOptions;
use std::io::{self, BufReader, BufWriter};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "Trackit")]
#[command(about = "A simple time tracker CLI tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start tracking a new task
    Start {
        /// Task name or description
        name: String,
    },
    /// Stop the current task
    Stop,
    /// Show log of tracked tasks
    Log,
}

#[derive(Serialize, Deserialize, Debug)]
struct Task {
    task: String,
    start_time: DateTime<Utc>,
    end_time: Option<DateTime<Utc>>,
}

const DB_FILE: &str = "trackit_log.json";

fn load_tasks() -> io::Result<Vec<Task>> {
    if !PathBuf::from(DB_FILE).exists() {
        return Ok(Vec::new());
    }

    let file = OpenOptions::new().read(true).open(DB_FILE)?;
    let reader = BufReader::new(file);
    let tasks = serde_json::from_reader(reader).unwrap_or_default();
    Ok(tasks)
}

fn save_tasks(tasks: &Vec<Task>) -> io::Result<()> {
    let file = OpenOptions::new().write(true).create(true).truncate(true).open(DB_FILE)?;
    let writer = BufWriter::new(file);
    serde_json::to_writer_pretty(writer, tasks)?;
    Ok(())
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let mut tasks = load_tasks()?;

    match &cli.command {
        Commands::Start { name } => {
            // If last task is running, stop it
            if let Some(last) = tasks.last_mut() {
                if last.end_time.is_none() {
                    last.end_time = Some(Utc::now());
                }
            }

            let new_task = Task {
                task: name.clone(),
                start_time: Utc::now(),
                end_time: None,
            };

            tasks.push(new_task);
            save_tasks(&tasks)?;
            println!("Started tracking: {}", name);
        }

        Commands::Stop => {
            if let Some(last) = tasks.last_mut() {
                if last.end_time.is_none() {
                    last.end_time = Some(Utc::now());

                    let task_name = last.task.clone(); // clone name
                    let _ = last; // release mutable borrow

                    save_tasks(&tasks)?;
                    println!("Stopped task: {}", task_name);
                } else {
                    println!("No task is currently running.");
                }
            } else {
                println!("No tasks found.");
            }
        }

        Commands::Log => {
            if tasks.is_empty() {
                println!("No tracked tasks yet.");
            } else {
                println!("Task History:");
                for (i, task) in tasks.iter().enumerate() {
                    let start = task.start_time.format("%Y-%m-%d %H:%M:%S");
                    let end = match task.end_time {
                        Some(end_time) => end_time.format("%Y-%m-%d %H:%M:%S").to_string(),
                        None => String::from("Ongoing"),
                    };
                    println!("{}. {} | Start: {} | End: {}", i + 1, task.task, start, end);
                }
            }
        }
    }

    Ok(())
}
