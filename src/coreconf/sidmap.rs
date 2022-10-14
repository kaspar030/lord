use std::collections::HashMap;

use super::sidfile::SidFile;

pub static SIDMAP: once_cell::sync::Lazy<SidMap> = once_cell::sync::Lazy::new(|| {
    let mut sidmap = crate::coreconf::SidMap::new();
    let project = directories::ProjectDirs::from("org", "CoreConfsters", "ccc").unwrap();

    let sid_glob = project.data_dir().join("*.sid");
    let sid_glob = sid_glob.to_string_lossy();

    for entry in glob::glob(&sid_glob).expect("bad data_dir sid glob pattern") {
        let entry = entry.expect("unexpected glob result");
        if let Err(e) = sidmap.add_file(&entry) {
            eprintln!("error: ccc: while reading {}: {}", entry.display(), e);
        }
    }

    sidmap
});

pub struct SidMap {
    pub sid2string: HashMap<usize, String>,
    pub string2sid: HashMap<String, usize>,
}

impl SidMap {
    pub fn new() -> Self {
        Self {
            sid2string: HashMap::new(),
            string2sid: HashMap::new(),
        }
    }

    pub fn add_file<T: AsRef<std::path::Path>>(
        &mut self,
        path: T,
    ) -> Result<&mut Self, Box<dyn std::error::Error>> {
        let mut sidfile = SidFile::from_file(path)?;
        self.sid2string.extend(
            sidfile
                .items
                .drain(..)
                .map(|item| (item.sid, Self::shorten_identifier(item.identifier))),
        );

        self.string2sid
            .extend(self.sid2string.iter().map(|(k, v)| (v.clone(), k.clone())));

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
