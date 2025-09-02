use html_escape::encode_text;
use mime_guess::from_path;
use std::{
    io::Cursor,
    iter::{Skip, Take},
    path::PathBuf,
    slice::Iter,
};
use tiny_http::{Header, Response};

static FILES_PER_PAGE: usize = 20;

pub struct Listing<'a> {
    paths: Take<Skip<Iter<'a, PathBuf>>>,
    total_paths: usize,
    page: usize,
    total_pages: usize,
}

impl<'a> Listing<'a> {
    pub fn new(paths: &'a [PathBuf], page: usize) -> Self {
        let total_paths = paths.len();
        let total_pages = (total_paths as f64 / FILES_PER_PAGE as f64).ceil() as usize;
        let paths = paths
            .iter()
            .skip((page.max(1) - 1) * FILES_PER_PAGE)
            .take(FILES_PER_PAGE);

        Self {
            paths,
            total_paths,
            page,
            total_pages,
        }
    }
}

impl From<Listing<'_>> for Response<Cursor<Vec<u8>>> {
    fn from(value: Listing) -> Self {
        let info_element = format!("<div>{} total files</div>", value.total_paths);

        let mut file_elements = String::new();

        for path in value.paths {
            let path_str = path.to_string_lossy();

            let mut elements = format!(
                r#"<a href="{path_str}" target="_blank">{}</a>"#,
                encode_text(path_str.split('/').next_back().unwrap_or_default(),),
            );

            let mime_type = from_path(path).first_or_octet_stream();

            match mime_type
                .essence_str()
                .split('/')
                .next()
                .unwrap_or_default()
            {
                tag @ ("audio" | "video") => {
                    elements += &format!(r#"<{tag} src="{path_str}" controls></{tag}>"#)
                }
                "image" => elements += &format!(r#"<img src="{path_str}" />"#),
                _ => elements += r#"<img />"#,
            }

            elements += &format!("<div>{mime_type}</div>");

            file_elements += &format!(r#"<div class="file">{elements}</div>"#);
        }

        let mut paginator_elements = String::new();

        for i in 1..=value.total_pages {
            if i == value.page {
                paginator_elements += &format!(r#"<a class="current">{i}</a>"#);
            } else {
                paginator_elements += &format!(r#"<a href="?page={i}">{i}</a>"#);
            }
        }

        let html = include_str!("listing.html")
            .replace("{info_element}", &info_element)
            .replace("{paginator_elements}", &paginator_elements)
            .replace("{file_elements}", &file_elements);

        let mut response = Response::from_string(html);

        if let Ok(header) = Header::from_bytes("content-type", "text/html") {
            response = response.with_header(header);
        }

        response
    }
}
