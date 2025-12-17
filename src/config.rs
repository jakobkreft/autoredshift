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
                ConfigPoint { hour: 4.1, temp: 3556 },
                ConfigPoint { hour: 5.8, temp: 5710 },
                ConfigPoint { hour: 7.4, temp: 6323 },
                ConfigPoint { hour: 12.8, temp: 6496 },
                ConfigPoint { hour: 17.6, temp: 6353 },
                ConfigPoint { hour: 19.3, temp: 5595 },
                ConfigPoint { hour: 21.5, temp: 3742 },
            ],
        }
    }
}

impl Config {
    pub fn load() -> Self {
        let path = Self::get_config_path();
        if let Some(path) = path {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(path) {
                    if let Ok(config) = serde_json::from_str(&content) {
                        return config;
                    }
                }
            }
        }
        Self::default()
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
