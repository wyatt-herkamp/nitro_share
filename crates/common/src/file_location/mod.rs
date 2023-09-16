use std::{io, path::PathBuf};

use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

pub trait GetWebPath {
    fn get_path(&self) -> String;

    fn get_url(&self, base_url: String) -> String {
        format!("{}/{}", base_url, self.get_path())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "sea-orm", derive(sea_orm::FromJsonQueryResult))]
pub enum FileLocation {
    /// Local file
    Local {
        /// Path to file
        location: PathBuf,
        /// Size of file.
        /// If the file changed directly, this size may be incorrect
        ///
        /// If you do decide to modify the file directly.
        ///
        /// Either run
        ///
        /// `nitro-share-admin update-file-size --type <paste|image> --id <id>`.
        ///
        /// `nitro-share-admin update-file-size --type <paste|image> --all`
        size: usize,
    },
}

impl FileLocation {
    pub fn new_local(location: PathBuf, size: usize) -> Self {
        Self::Local { location, size }
    }
    pub fn file_size(&self) -> usize {
        match self {
            Self::Local { size, .. } => *size,
        }
    }

    pub fn get_hash_files(&self) -> Vec<HashFile> {
        match self {
            Self::Local { .. } => {
                let hash_files = Vec::new();
                hash_files
            }
        }
    }
    pub fn build_hash_file(&self, _hash_type: HashFileType) -> Result<HashFile, io::Error> {
        todo!()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum HashFileType {
    MD5,
    SHA256,
}
impl HashFileType {
    pub fn get_hash(&self, _content: &[u8]) -> Vec<u8> {
        match self {
            _ => unimplemented!("HashFileType::get_hash"),
        }
    }
}
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct HashFile {
    pub hash_type: HashFileType,
    #[serde(skip)]
    pub file: FileLocation,
    pub content: Vec<u8>,
}
