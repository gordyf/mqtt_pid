use std::io::{BufRead, BufReader};

use clap::Parser;
use jiff::{Timestamp, ToSpan};
use pid::Pid;

#[derive(Parser)]
pub struct SimulateArgs {
    #[arg(short, long)]
    input_file: String,
}

struct Event {
    value: f64,
    timestamp: Timestamp,
}

impl SimulateArgs {
    pub async fn run(
        self: SimulateArgs,
        mut pid: Pid<f64>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Read the CSV file
        let mut events: Vec<Event> = Vec::new();
        let file = std::fs::File::open(self.input_file)?;
        let reader = BufReader::new(file);

        for line_result in reader.lines() {
            let line = line_result?;
            let parts: Vec<&str> = line.trim().split(',').collect();
            if parts.len() != 2 {
                continue; // Skip invalid lines
            }
            let value = parts[0].parse()?;
            let timestamp_str = parts[1];
            events.push(Event {
                value,
                timestamp: timestamp_str.parse()?,
            });
        }

        // Sort events by timestamp
        events.sort_by_key(|e| e.timestamp);

        if events.is_empty() {
            return Ok(());
        }

        // Determine start and end times
        let first_event = events.first().unwrap();
        let last_event = events.last().unwrap();

        let start_time = first_event.timestamp;
        let end_time = last_event.timestamp + 1.seconds();

        let mut current_value = 0.0;
        let mut event_index = 0;

        let mut t_s = start_time;

        while t_s < end_time {
            // Update current_value based on events
            while event_index < events.len() && events[event_index].timestamp <= t_s {
                current_value = events[event_index].value;
                event_index += 1;
            }

            let output = pid.next_control_output(current_value).output;
            let output = output.clamp(0.0, 100.0);
            println!("{},{},{}", t_s.as_second(), current_value, output);

            // Increment time by one second
            t_s += 1.seconds();
        }

        Ok(())
    }
}
