use crate::{config::Config, listing::Listing};
use anyhow::{Context, Error, Result, bail};
use mime_guess::from_path;
use natord::compare_ignore_case;
use rand::{Rng, rng};
use std::{
    fmt::Display,
    fs::{File, read_dir},
    net::{Ipv4Addr, SocketAddrV4},
    path::PathBuf,
    time::{SystemTime, UNIX_EPOCH},
};
use tiny_http::{Header, Request, Response, Server};

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
            "Random File Server started! Port: {}, Cache TTL: {}s, Non-Repeat: {}, Listing Path: {}",
            self.config.port,
            self.config.cache_ttl_secs,
            self.config.non_repeat,
            self.config.listing_path.as_ref().map_or_else(
                || "Disabled".into(),
                |listing_path| format!("/{}", listing_path.trim_matches('/')),
            ),
        );

        for request in server.incoming_requests() {
            // Ignore /favicon.ico requests
            if request.url().starts_with("/favicon.ico") {
                continue;
            }

            if let Err(error) = self.refresh_paths() {
                println!("Error while refreshing paths: {error}");
            }

            if let Err(error) = self.handle(request) {
                println!("Error while processing request: {error}");
            }
        }

        Ok(())
    }

    fn handle(&mut self, request: Request) -> Result<()> {
        if let Some(listing_path) = &self.config.listing_path {
            if request.url().starts_with("/files/") {
                let path = request
                    .url()
                    .trim_matches('/')
                    .replace("%20", " ")
                    .replace('+', "");

                if let Ok((file, header)) = self.get_file(path) {
                    let response = Response::from_file(file).with_header(header);
                    request.respond(response)?;
                }

                return Ok(());
            }

            if request
                .url()
                .split('?')
                .next()
                .unwrap_or_default()
                .trim_matches('/')
                == listing_path.trim_matches('/')
            {
                let url_params = request.url().split('/').next_back().unwrap_or_default();
                let page_split = url_params.split_once("page=").unwrap_or_default();
                let page = page_split
                    .1
                    .split('&')
                    .next()
                    .unwrap_or_default()
                    .parse::<usize>()
                    .unwrap_or(1);

                let response = Listing::new(&self.paths, page).into();
                request.respond(response)?;

                return Ok(());
            }
        }

        if let Ok((file, header)) = self.get_random_file() {
            let response = Response::from_file(file).with_header(header);
            request.respond(response)?;
        }

        Ok(())
    }

    fn get_file<T: Display>(&mut self, path: T) -> Result<(File, Header)> {
        let path = path.to_string();
        let path = self
            .paths
            .iter()
            .find(|entry| entry.to_string_lossy() == path)
            .context("File not found")?;

        let file = File::open(path)?;
        let content_type = from_path(path).first_or_octet_stream().to_string();
        let header = Header::from_bytes("content-type", content_type)
            .map_err(|_| Error::msg("Could not create header"))?;

        Ok((file, header))
    }

    fn get_random_file(&mut self) -> Result<(File, Header)> {
        let path = if self.config.non_repeat {
            let paths_unused = self
                .paths
                .iter()
                .filter(|path| !self.paths_used.contains(path))
                .collect::<Vec<&PathBuf>>();

            if paths_unused.is_empty() {
                self.paths_used = vec![];
                &self.paths[rng().random_range(0..self.paths.len())]
            } else {
                paths_unused[rng().random_range(0..paths_unused.len())]
            }
        } else {
            &self.paths[rng().random_range(0..self.paths.len())]
        };

        if self.config.non_repeat {
            self.paths_used.push(path.clone());
        }

        let file = File::open(path)?;
        let content_type = from_path(path).first_or_octet_stream().to_string();
        let header = Header::from_bytes("content-type", content_type)
            .map_err(|_| Error::msg("Could not create header"))?;

        Ok((file, header))
    }

    fn refresh_paths(&mut self) -> Result<()> {
        if Self::get_current_timestamp() < self.paths_last_updated + self.config.cache_ttl_secs {
            return Ok(());
        }

        self.paths.clear();

        for entry in read_dir("files")? {
            let Ok(entry) = entry else { continue };
            let is_file = entry.file_type().is_ok_and(|file_type| file_type.is_file());

            if is_file {
                self.paths.push(entry.path());
            }
        }

        if self.paths.is_empty() {
            bail!("No paths found");
        }

        self.paths
            .sort_by(|a, b| compare_ignore_case(&b.to_string_lossy(), &a.to_string_lossy()));

        self.paths_last_updated = Self::get_current_timestamp();

        Ok(())
    }

    fn get_current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
