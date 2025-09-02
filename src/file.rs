use anyhow::{Error, Result};
use mime_guess::{Mime, from_path};
use std::{fs::File as FsFile, path::PathBuf};
use tiny_http::{Header, Response};

#[derive(Clone)]
pub struct File {
    pub path: PathBuf,
    pub mime: Mime,
    pub size: u64,
}

impl File {
    pub fn new(path: PathBuf) -> Result<Self> {
        let mime = from_path(&path).first_or_octet_stream();
        let metadata = FsFile::open(&path).and_then(|file| file.metadata())?;

        Ok(Self {
            path,
            mime,
            size: metadata.len(),
        })
    }

    pub fn get_response(&self) -> Result<Response<FsFile>> {
        let fs_file = FsFile::open(&self.path)?;
        let header = Header::from_bytes("content-type", self.mime.essence_str())
            .map_err(|_| Error::msg("Could not create header"))?;
        let response = Response::from_file(fs_file).with_header(header);

        Ok(response)
    }
}

impl PartialEq for File {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}
