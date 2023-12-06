use mime_guess::from_path;
use rand::{thread_rng, Rng};
use std::{
    env::var,
    fs::{read_dir, File},
    net::{Ipv4Addr, SocketAddrV4},
    path::PathBuf,
};
use tiny_http::{Header, Response, Server};

fn main() {
    let server = Server::http(SocketAddrV4::new(
        Ipv4Addr::new(0, 0, 0, 0),
        var("PORT")
            .ok()
            .and_then(|port| port.parse::<u16>().ok())
            .unwrap_or(8000),
    ))
    .expect("Could not start server");

    for request in server.incoming_requests() {
        let Some((content_type, file)) = get_random_file() else {
            continue;
        };

        let _ = request.respond(Response::from_file(file).with_header(
            Header::from_bytes("content-type", content_type).expect("Could not add header"),
        ));
    }
}

pub fn get_random_file() -> Option<(String, File)> {
    let Some(path) = get_random_path() else {
        return None;
    };

    let Ok(file) = File::open(&path) else {
        return None;
    };

    Some((from_path(path).first_or_octet_stream().to_string(), file))
}

fn get_random_path() -> Option<PathBuf> {
    let mut paths = vec![];

    let Ok(dir) = read_dir("files") else {
        return None;
    };

    for entry in dir {
        let Ok(entry) = entry else { continue };

        if entry
            .file_type()
            .map_or(false, |file_type| file_type.is_file())
        {
            paths.push(entry.path());
        }
    }

    if paths.is_empty() {
        None
    } else {
        Some(paths.remove(thread_rng().gen_range(0..paths.len())))
    }
}
