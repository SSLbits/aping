use std::io::Write;
use std::sync::{Arc, atomic::{AtomicBool, Ordering}};
use std::time::Duration;
use std::{io, thread};
use clap::{Arg, Command};
use std::process::Command as ProcessCommand;
use crossterm::event::{self, Event, KeyCode};

fn main() {
    let matches = Command::new("aping")
        .version("1.0")
        .about("Audible ping application")
        .arg(
            Arg::new("destination")
                .help("The target host or IP address to ping")
                .required(true),
        )
        .arg(
            Arg::new("inverse")
                .short('i')
                .long("inverse")
                .help("Beep on failed pings instead of successful ones")
                .takes_value(false),
        )
        .get_matches();

    let destination = matches.get_one::<String>("destination").unwrap();
    let inverse_mode = matches.is_present("inverse");

    // Atomic boolean to handle graceful shutdown
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();

    // Set up Ctrl+C handler
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })
    .expect("Error setting Ctrl+C handler");

    // Spawn thread to listen for 'q' keypress
    let r = running.clone();
    thread::spawn(move || {
        while r.load(Ordering::SeqCst) {
            if event::poll(Duration::from_millis(100)).unwrap() {
                if let Event::Key(key) = event::read().unwrap() {
                    if key.code == KeyCode::Char('q') {
                        r.store(false, Ordering::SeqCst);
                        break;
                    }
                }
            }
        }
    });

    println!("Pinging {}... Press 'q' to quit.", destination);

    let mut sent = 0;
    let mut received = 0;
    let mut total_time = 0;
    let mut min_time = std::i32::MAX;
    let mut max_time = std::i32::MIN;

    while running.load(Ordering::SeqCst) {
        let output = if cfg!(target_os = "windows") {
            ProcessCommand::new("ping")
                .arg("-n")
                .arg("1")
                .arg(destination)
                .output()
        } else {
            ProcessCommand::new("ping")
                .arg("-c")
                .arg("1")
                .arg(destination)
                .output()
        };

        match output {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output.status.success() {
                    sent += 1;
                    if cfg!(target_os = "windows") {
                        if let Some(line) = output_str.lines().find(|line| line.contains("time=") || line.contains("time<1ms")) {
                            println!("{}", line.trim());
                            if line.contains("time<1ms") {
                                received += 1;
                                total_time += 1;
                                if 1 < min_time {
                                    min_time = 1;
                                }
                                if 1 > max_time {
                                    max_time = 1;
                                }
                            } else if let Some(time_str) = line.split("time=").nth(1) {
                                if let Some(time) = time_str.split("ms").next() {
                                    if let Ok(time) = time.trim().parse::<i32>() {
                                        received += 1;
                                        total_time += time;
                                        if time < min_time {
                                            min_time = time;
                                        }
                                        if time > max_time {
                                            max_time = time;
                                        }
                                    }
                                }
                            }
                        } else {
                            println!("Unexpected output: {}", output_str);
                        }
                    } else {
                        if let Some(line) = output_str.lines().find(|line| line.contains("icmp_seq")) {
                            println!("{}", line.trim());
                            if let Some(time_str) = line.split("time=").nth(1) {
                                if let Some(time) = time_str.split(" ms").next() {
                                    if let Ok(time) = time.trim().parse::<i32>() {
                                        received += 1;
                                        total_time += time;
                                        if time < min_time {
                                            min_time = time;
                                        }
                                        if time > max_time {
                                            max_time = time;
                                        }
                                    }
                                }
                            }
                        } else {
                            println!("Unexpected output: {}", output_str);
                        }
                    }
                    if !inverse_mode {
                        print!("\x07"); // Audible beep
                        io::stdout().flush().unwrap();
                    }
                } else {
                    println!("Ping failed to {}", destination);
                    println!("Output: {}", output_str);
                    if inverse_mode {
                        print!("\x07"); // Audible beep
                        io::stdout().flush().unwrap();
                    }
                }
            }
            Err(err) => {
                println!("Error executing ping: {}", err);
                if inverse_mode {
                    print!("\x07"); // Audible beep
                    io::stdout().flush().unwrap();
                }
            }
        }

        thread::sleep(Duration::from_secs(1)); // Wait 1 second before next ping
    }

    println!("Exiting aping.");
    println!("\nPing statistics for {}:", destination);
    println!("    Packets: Sent = {}, Received = {}, Lost = {} ({}% loss),", sent, received, sent - received, ((sent - received) as f64 / sent as f64) * 100.0);
    if received > 0 {
        println!("Approximate round trip times in milli-seconds:");
        println!("    Minimum = {}ms, Maximum = {}ms, Average = {}ms", min_time, max_time, total_time / received);
    }
}