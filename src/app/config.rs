use anyhow::Result;
use home::home_dir;
use std::{collections::HashMap, io::{Read, Write}, path::PathBuf};
use serde_json::{from_str, to_string_pretty as to_string};
use serde::{Serialize, Deserialize};

#[derive(Debug, Default, Serialize, Deserialize)]
struct RegionConfig{
    hidden: bool,
    favorite: bool
}

// when it becomes stable as const , switch to Duration::from_days(7).as_secs();
// https://github.com/rust-lang/rust/issues/120301
const DEFAULT_RECENT_TIMEOUT: u64 = 60 * 60 * 24 * 7; 
#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Config {
    recent_timeout: u64,
    regions: HashMap<String, RegionConfig>,
}

impl Default for Config {
    fn default() -> Self {
        let mut regions = HashMap::new();
        for region in DEFAULT_REGIONS {
            regions.insert(region.to_string(), RegionConfig::default());
        }
        Config {
            regions,
            recent_timeout: DEFAULT_RECENT_TIMEOUT,
        }
    }
}

const DEFAULT_REGIONS: &[&str] = &[
    "af-south-1",
    "ap-east-1",
    "ap-northeast-1",
    "ap-northeast-2",
    "ap-northeast-3",
    "ap-south-1",
    "ap-south-2",
    "ap-southeast-1",
    "ap-southeast-2",
    "ap-southeast-3",
    "ap-southeast-4",
    "ca-central-1",
    "ca-west-1",
    "eu-central-1",
    "eu-central-2",
    "eu-north-1",
    "eu-south-1",
    "eu-south-2",
    "eu-west-1",
    "eu-west-2",
    "eu-west-3",
    "il-central-1",
    "me-central-1",
    "me-south-1",
    "sa-east-1",
    "us-east-1",
    "us-east-2",
    "us-west-1",
    "us-west-2",
];

impl Config {
    pub fn new() -> Result<Config> {
       let config_path = Config::get_config_path()?;

       let mut file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(false)
            .read(true)
            .write(true)
            .open(config_path)?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let config = match from_str(&contents) {
            Ok(config) => config,
            Err(_) => {
                let config = Config::default();
                config.persist()?;
                config
            },
        };
        Ok(config)

    }

    pub fn persist(&self) -> Result<()> {
        let config_path = Config::get_config_path()?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .truncate(true)
            .open(config_path)?;
        file.write_all(to_string(&self)?.as_bytes())?;
        Ok(())
    }

    fn get_config_path() -> Result<PathBuf> {
        let Some(home_dir) = home_dir() else {
            return Result::Err(anyhow::anyhow!("Could not find home directory"));
        };
        Ok(home_dir.join(".sm_connect.json"))
    }

    fn get_default_regions() -> Vec<String> {
        DEFAULT_REGIONS.iter().map(|s| s.to_string()).collect()
    }

    pub fn get_visible_regions(&self) -> Vec<String> {
        self.regions
            .iter()
            .filter(|(_, region)| !region.hidden)
            .map(|(region, _)| region.to_string())
            .collect()
    }

    pub fn hide_region(&mut self, region: String) -> Result<()> {
        if let Some(region) = self.regions.get_mut(&region) {
            region.hidden = true;
        }
        self.persist()
    }

    pub fn reset_hidden_regions(&mut self) -> Result<()> {
        for region in self.regions.values_mut() {
            region.hidden = false;
        }
        self.persist()
    }

    pub fn get_favorite_regions(&self) -> Vec<String> {
        self.regions
            .iter()
            .filter(|(_, region)| region.favorite)
            .map(|(region, _)| region.to_string())
            .collect()
    }

    pub fn toggle_favorite_region(&mut self, region: String) -> Result<()> {
        if let Some(region) = self.regions.get_mut(&region) {
            region.favorite = !region.favorite;
        }
        self.persist()
    }

    pub fn unset_favorite_region(&mut self, region: String) -> Result<()> {
        if let Some(region) = self.regions.get_mut(&region) {
            region.favorite = false;
        }
        self.persist()
    }

    pub fn favorite_region(&mut self, region: String) -> Result<()> {
        if let Some(region) = self.regions.get_mut(&region) {
            region.favorite = true;
        }
        self.persist()
    }
    
    pub fn get_recent_timeout(&self) -> u64 {
        self.recent_timeout
    }

    pub fn set_recent_timeout(&mut self, timeout: u64) -> Result<()> {
        self.recent_timeout = timeout;
        self.persist()
    }
}
