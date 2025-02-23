use std::{collections::HashMap, hash::Hash, io::BufRead, path::PathBuf};
use anyhow::Result;
use home::home_dir;
use std::io::Write;
use serde::{Serialize, Deserialize};
use serde_json::{from_str, to_string};
use std::time::SystemTime;

pub fn get_current_time() -> u64{
    SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs()
}

pub struct History {}

#[derive(Debug,Serialize,Deserialize)]
pub struct HistoryEntry {
    instance_id: String,
    region: String,
    when: u64
}

impl HistoryEntry {
    pub fn new(instance_id: String, region: String) -> HistoryEntry {
        HistoryEntry {
            instance_id,
            region,
            when: get_current_time()
        }
    }

    pub fn get_instance_id(&self) -> &str {
        &self.instance_id
    }

    pub fn get_when(&self) -> u64 {
        self.when
    }
}

impl History {
    pub fn save(entry: HistoryEntry) -> Result<()> {
        let file = Self::get_history_path()?;
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(file)
            .unwrap();
        writeln!(file, "{}", to_string(&entry).unwrap()).unwrap();
        file.flush()?;
        Ok(())
    }

    pub fn read() -> Result<HashMap<String,HistoryEntry>> {
        //TOOD: Persist the deduplicated entries?
        let mut entries  = HashMap::new();
        let file = Self::get_history_path()?;
        let Ok(file) = std::fs::File::open(file) else {
            return Ok(entries);
        };
        
        let reader = std::io::BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let entry: HistoryEntry = from_str(&line)?;
            match entries.get_mut(entry.get_instance_id()) {
                Some(existing) => {
                    if existing.when < entry.when {
                        *existing = entry;
                    }
                }
                None => {
                    entries.insert(entry.get_instance_id().to_string(), entry);
                }
            }
        }
        Ok(entries)
    }

    fn get_history_path() -> Result<PathBuf> {
        let Some(home_dir) = home_dir() else {
            return Result::Err(anyhow::anyhow!("Could not find home directory"));
        };
        Ok(home_dir.join(".sm_connect_history"))
    }
}