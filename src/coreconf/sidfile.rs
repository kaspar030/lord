use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct SidFile {
    #[serde(rename = "assignment-ranges")]
    pub assignment_ranges: Vec<AssignmentRange>,
    #[serde(rename = "module-name")]
    pub module_name: String,
    #[serde(rename = "module-revision")]
    pub module_revision: String,
    pub items: Vec<Item>,
}

impl SidFile {
    pub fn from_file<T: AsRef<std::path::Path>>(
        path: T,
    ) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&content[..])?)
    }
}

#[derive(Serialize, Deserialize)]
pub struct AssignmentRange {
    #[serde(rename = "entry-point")]
    pub entry_point: usize,
    pub size: usize,
}

#[derive(Serialize, Deserialize)]
pub struct Item {
    pub namespace: String,
    pub identifier: String,
    pub sid: usize,
}
