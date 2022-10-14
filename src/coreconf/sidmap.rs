use std::collections::HashMap;

use super::sidfile::SidFile;

pub struct SidMap {
    pub map: HashMap<usize, String>,
}

impl SidMap {
    pub fn new() -> Self {
        Self {
            map: HashMap::new(),
        }
    }

    pub fn add_file<T: AsRef<std::path::Path>>(
        &mut self,
        path: T,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let mut sidfile = SidFile::from_file(path)?;
        self.map.extend(
            sidfile
                .items
                .drain(..)
                .map(|item| (item.sid, Self::shorten_identifier(item.identifier))),
        );
        Ok(self)
    }

    fn shorten_identifier(s: String) -> String {
        std::path::PathBuf::from(s)
            .file_name()
            .unwrap()
            .to_string_lossy()
            .to_string()
    }
}
