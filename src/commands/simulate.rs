use std::io::{BufRead, BufReader};

use chrono::{DateTime, Duration, TimeZone, Timelike, Utc};
use clap::Parser;
use pid::Pid;

#[derive(Parser, Clone)]
pub struct SimulateArgs {
    #[arg(short, long)]
    input_file: String,
}

struct Event {
    value: f32,
    timestamp: DateTime<Utc>,
}

impl SimulateArgs {
    pub async fn run(
        self: SimulateArgs,
        mut pid: Pid<f32>,
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
            let value: f32 = parts[0].parse()?;
            let timestamp_str = parts[1];
            let timestamp = timestamp_str.parse::<DateTime<Utc>>()?;
            events.push(Event { value, timestamp });
        }

        // Sort events by timestamp
        events.sort_by_key(|e| e.timestamp);

        if events.is_empty() {
            return Ok(());
        }

        // Determine start and end times
        let first_event = events.first().unwrap();
        let last_event = events.last().unwrap();

        let start_time = first_event
            .timestamp
            .date_naive()
            .and_hms_opt(
                first_event.timestamp.hour(),
                first_event.timestamp.minute(),
                first_event.timestamp.second(),
            )
            .unwrap();

        let mut end_time = last_event
            .timestamp
            .date_naive()
            .and_hms_opt(
                last_event.timestamp.hour(),
                last_event.timestamp.minute(),
                last_event.timestamp.second(),
            )
            .unwrap();

        // Include the last second
        end_time += Duration::seconds(1);

        let mut current_value = 0.0;
        let mut event_index = 0;

        let mut t_s = start_time;

        while t_s < end_time {
            // Update current_value based on events
            while event_index < events.len()
                && events[event_index].timestamp <= Utc.from_utc_datetime(&t_s)
            {
                current_value = events[event_index].value;
                event_index += 1;
            }

            let output = pid.next_control_output(current_value).output;
            let output = output.clamp(0.0, 100.0);
            println!("{},{},{}", t_s, current_value, output);

            // Increment time by one second
            t_s += Duration::seconds(1);
        }

        Ok(())
    }
}
