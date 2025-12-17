use serde::{Deserialize, Serialize};
use splines::{Interpolation, Key, Spline};
use std::fs;
use std::path::PathBuf;
use directories::ProjectDirs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigPoint {
    pub hour: f32,
    pub temp: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub points: Vec<ConfigPoint>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            points: vec![
                ConfigPoint { hour: 3.0, temp: 3455 },
                ConfigPoint { hour: 5.0, temp: 4601 },
                ConfigPoint { hour: 6.0, temp: 6137 },
                ConfigPoint { hour: 7.0, temp: 6468 },
                ConfigPoint { hour: 15.0, temp: 6082 },
                ConfigPoint { hour: 22.0, temp: 3102 },
            ],
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = match Self::get_config_path() {
            Some(path) => path,
            None => {
                eprintln!(
                    "Could not determine config directory. Using defaults. Run `autoredshift --config` to create one."
                );
                return Self::default();
            }
        };

        if !path.exists() {
            eprintln!(
                "No config found at {}. Using defaults. Run `autoredshift --config` to create it.",
                path.display()
            );
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(content) => match serde_json::from_str(&content) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!(
                        "Failed to parse config at {}: {}. Using defaults. Run `autoredshift --config` to recreate it.",
                        path.display(),
                        e
                    );
                    Self::default()
                }
            },
            Err(e) => {
                eprintln!(
                    "Failed to read config at {}: {}. Using defaults. Run `autoredshift --config` to recreate it.",
                    path.display(),
                    e
                );
                Self::default()
            }
        }
    }

    pub fn save(&self) -> anyhow::Result<()> {
        let path = Self::get_config_path().ok_or_else(|| anyhow::anyhow!("Could not determine config path"))?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    fn get_config_path() -> Option<PathBuf> {
        ProjectDirs::from("com", "autoredshift", "autoredshift")
            .map(|proj| proj.config_dir().join("config.json"))
    }

    pub fn get_temperature(&self, hour: f32) -> u32 {
        // Create ghost points for circular interpolation
        let mut keys = Vec::new();
        
        for p in &self.points {
            keys.push(Key::new(p.hour - 24.0, p.temp as f32, Interpolation::CatmullRom));
            keys.push(Key::new(p.hour, p.temp as f32, Interpolation::CatmullRom));
            keys.push(Key::new(p.hour + 24.0, p.temp as f32, Interpolation::CatmullRom));
        }
        
        // Sort keys by hour
        keys.sort_by(|a, b| a.t.partial_cmp(&b.t).unwrap());

        let spline = Spline::from_vec(keys);
        
        // Clamp hour to 0-24 just in case
        let hour = hour.clamp(0.0, 24.0);
        
        spline.sample(hour).unwrap_or(1000.0).clamp(1000.0, 25000.0) as u32
    }
}
