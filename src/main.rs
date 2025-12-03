mod config;
mod gui;

use clap::Parser;
use chrono::{Local, Timelike};
use std::process::Command;
use crate::config::Config;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Open configuration GUI
    #[arg(short, long)]
    config: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    if args.config {
        let options = eframe::NativeOptions {
            viewport: eframe::egui::ViewportBuilder::default()
                .with_inner_size([800.0, 600.0]),
            ..Default::default()
        };
        eframe::run_native(
            "Autoredshift Config",
            options,
            Box::new(|cc| Ok(Box::new(gui::App::new(cc)))),
        ).map_err(|e| anyhow::anyhow!("GUI Error: {}", e))?;
    } else {
        let config = Config::load();
        let now = Local::now();
        
        println!("Current time: {}", now.format("%H:%M:%S"));
        // Calculate decimal hour
        let hour = now.hour() as f32 + now.minute() as f32 / 60.0 + now.second() as f32 / 3600.0;
        
        let temp = config.get_temperature(hour);
        
        println!("Calculated temperature: {}K", temp);
        
        // Run redshift
        // redshift -P -O <temp>
        let status = Command::new("redshift")
            .arg("-P")
            .arg("-O")
            .arg(temp.to_string())
            .status();

        match status {
            Ok(s) => {
                if !s.success() {
                    eprintln!("Redshift exited with error code: {:?}", s.code());
                }
            }
            Err(e) => {
                eprintln!("Failed to execute redshift: {}", e);
                eprintln!("Make sure 'redshift' is installed and in your PATH.");
            }
        }
    }

    Ok(())
}
