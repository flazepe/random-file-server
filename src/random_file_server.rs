use crate::{config::Config, file::File, listing::Listing, traits::Commas};
use anyhow::{Context, Error, Result, bail};
use natord::compare_ignore_case;
use rand::{Rng, rng};
use std::{
    fs::{File as FsFile, read_dir},
    net::{Ipv4Addr, SocketAddrV4},
    time::{SystemTime, UNIX_EPOCH},
};
use tiny_http::{Request, Response, Server};

pub struct RandomFileServer {
    config: Config,
    files: Vec<File>,
    files_used: Vec<File>,
    files_last_updated: u64,
}

impl RandomFileServer {
    pub fn new() -> Self {
        Self {
            config: Config::get(),
            files: vec![],
            files_used: vec![],
            files_last_updated: 0,
        }
    }

    pub fn start(&mut self) -> Result<()> {
        let addr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), self.config.port);
        let server = Server::http(addr).map_err(Error::msg)?;

        println!(
            "Random File Server started! Port: {}, Cache TTL: {}s, Non-Repeat: {}, Listing Path: {}",
            self.config.port,
            self.config.cache_ttl_secs.commas(),
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

            if let Err(error) = self.refresh_files() {
                println!("Error while refreshing files: {error}");
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
                return self.handle_file(request);
            }

            if request
                .url()
                .split('?')
                .next()
                .unwrap_or_default()
                .trim_matches('/')
                == listing_path.trim_matches('/')
            {
                return self.handle_file_listing(request);
            }
        }

        self.handle_random_file(request)
    }

    fn handle_file(&self, request: Request) -> Result<()> {
        let path = request
            .url()
            .trim_matches('/')
            .replace("%20", " ")
            .replace('+', "");

        if let Ok(response) = self.get_file_response(&path) {
            request.respond(response)?;
        }

        Ok(())
    }

    fn handle_file_listing(&self, request: Request) -> Result<()> {
        let url_params = request.url().split('/').next_back().unwrap_or_default();
        let page_split = url_params.split_once("page=").unwrap_or_default();
        let page = page_split
            .1
            .split('&')
            .next()
            .unwrap_or_default()
            .parse::<usize>()
            .unwrap_or(1);

        let response = Listing::new(&self.files, page).into();
        request.respond(response)?;

        Ok(())
    }

    fn handle_random_file(&mut self, request: Request) -> Result<()> {
        if let Ok(response) = self.get_random_file_response() {
            request.respond(response)?;
        }

        Ok(())
    }

    fn get_file_response(&self, path: &str) -> Result<Response<FsFile>> {
        self.files
            .iter()
            .find(|entry| entry.path.to_string_lossy() == path)
            .context("File not found")?
            .get_response()
    }

    fn get_random_file_response(&mut self) -> Result<Response<FsFile>> {
        let file = if self.config.non_repeat {
            let files_unused = self
                .files
                .iter()
                .filter(|path| !self.files_used.contains(path))
                .collect::<Vec<&File>>();

            if files_unused.is_empty() {
                self.files_used = vec![];
                &self.files[rng().random_range(0..self.files.len())]
            } else {
                files_unused[rng().random_range(0..files_unused.len())]
            }
        } else {
            &self.files[rng().random_range(0..self.files.len())]
        };

        if self.config.non_repeat {
            self.files_used.push(file.clone());
        }

        file.get_response()
    }

    fn refresh_files(&mut self) -> Result<()> {
        if Self::get_current_timestamp() < self.files_last_updated + self.config.cache_ttl_secs {
            return Ok(());
        }

        self.files.clear();

        for entry in read_dir("files")? {
            let Ok(entry) = entry else { continue };
            let is_file = entry.file_type().is_ok_and(|file_type| file_type.is_file());

            if is_file && let Ok(file) = File::new(entry.path()) {
                self.files.push(file);
            }
        }

        if self.files.is_empty() {
            bail!("No files found");
        }

        self.files.sort_by(|a, b| {
            compare_ignore_case(&b.path.to_string_lossy(), &a.path.to_string_lossy())
        });

        self.files_last_updated = Self::get_current_timestamp();

        Ok(())
    }

    fn get_current_timestamp() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
    }
}
