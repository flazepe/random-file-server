use crate::config::Config;
use anyhow::{Result, bail};
use mime_guess::from_path;
use rand::{Rng, rng};
use std::{
    fs::{File, read_dir},
    net::{Ipv4Addr, SocketAddrV4},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tiny_http::{Header, Response, Server};

pub struct RandomFileServer {
    config: Config,
    paths: Vec<PathBuf>,
    paths_used: Vec<PathBuf>,
    paths_last_updated: u64,
}

impl RandomFileServer {
    pub fn new() -> Self {
        Self {
            config: Config::get(),
            paths: vec![],
            paths_used: vec![],
            paths_last_updated: 0,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        let Ok(server) = Server::http(SocketAddrV4::new(
            Ipv4Addr::new(0, 0, 0, 0),
            self.config.port,
        )) else {
            bail!("Could not create server");
        };

        println!(
            "Random File Server started! Port: {}, Cache TTL: {}s, Non-Repeat: {}",
            self.config.port, self.config.cache_ttl_secs, self.config.non_repeat,
        );

        for request in server.incoming_requests() {
            // Ignore /favicon.ico requests
            if request.url().starts_with("/favicon.ico") {
                continue;
            }

            match self.get_random_file() {
                Ok((file, content_type)) => {
                    let Ok(header) = Header::from_bytes("content-type", content_type) else {
                        bail!("Could not create header");
                    };
                    let response = Response::from_file(file).with_header(header);
                    let _ = request.respond(response);
                }
                Err(error) => println!("Error while processing request: {error}"),
            }
        }

        Ok(())
    }

    fn get_random_file(&mut self) -> Result<(File, String)> {
        let mut paths = if self.config.non_repeat {
            let paths_unused = self
                .get_paths()?
                .into_iter()
                .filter(|path| !self.paths_used.contains(path))
                .collect::<Vec<PathBuf>>();

            if paths_unused.is_empty() {
                self.paths_used = vec![];
                self.get_paths()?
            } else {
                paths_unused
            }
        } else {
            self.get_paths()?
        };

        let path = paths.remove(rng().random_range(0..paths.len()));
        let file = File::open(&path)?;
        let content_type = from_path(&path).first_or_octet_stream().to_string();

        if self.config.non_repeat {
            self.paths_used.push(path);
        }

        Ok((file, content_type))
    }

    fn get_paths(&mut self) -> Result<Vec<PathBuf>> {
        if Self::get_current_timestamp() < self.paths_last_updated + self.config.cache_ttl_secs {
            return Ok(self.paths.clone());
        }

        let mut paths = vec![];

        for entry in read_dir("files")? {
            let Ok(entry) = entry else { continue };
            let is_file = entry.file_type().is_ok_and(|file_type| file_type.is_file());

            if is_file {
                paths.push(entry.path());
            }
        }

        if paths.is_empty() {
            bail!("No paths found");
        }

        self.paths = paths.clone();
        self.paths_last_updated = Self::get_current_timestamp();

        Ok(paths)
    }

    fn get_current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
