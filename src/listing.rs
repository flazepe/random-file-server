use crate::{file::File, traits::Commas};
use html_escape::encode_text;
use humansize::{DECIMAL, format_size};
use std::{
    io::Cursor,
    iter::{Skip, Take},
    slice::Iter,
};
use tiny_http::{Header, Response};

static FILES_PER_PAGE: usize = 20;

pub struct Listing<'a> {
    files: Take<Skip<Iter<'a, File>>>,
    total_files: usize,
    total_size: u64,
    total_pages: usize,
    page: usize,
}

impl<'a> Listing<'a> {
    pub fn new(files: &'a [File], page: usize) -> Self {
        let total_files = files.len();
        let total_size = files.iter().fold(0, |acc, file| acc + file.size);
        let total_pages = (total_files as f64 / FILES_PER_PAGE as f64).ceil() as usize;
        let files = files
            .iter()
            .skip((page.max(1) - 1) * FILES_PER_PAGE)
            .take(FILES_PER_PAGE);

        Self {
            files,
            total_files,
            total_size,
            total_pages,
            page,
        }
    }
}

impl From<Listing<'_>> for Response<Cursor<Vec<u8>>> {
    fn from(value: Listing) -> Self {
        let info_element = format!(
            "<div>{} total files - {}</div>",
            value.total_files.commas(),
            format_size(value.total_size, DECIMAL),
        );

        let mut paginator_elements = String::new();

        for page in 1..=value.total_pages {
            if page == value.page {
                paginator_elements += &format!(r#"<a class="current">{page}</a>"#);
            } else {
                paginator_elements += &format!(r#"<a href="?page={page}">{page}</a>"#);
            }
        }

        let mut file_elements = String::new();

        for file in value.files {
            let path_str = file.path.to_string_lossy();

            let mut elements = format!(
                r#"<a href="{path_str}" target="_blank">{}</a>"#,
                file.path.file_name().unwrap_or_default().display(),
            );

            match file.mime.type_().as_str() {
                tag @ ("audio" | "video") => {
                    elements += &format!(r#"<{tag} src="{path_str}" controls></{tag}>"#)
                }
                "image" => elements += &format!(r#"<img src="{path_str}" />"#),
                _ => elements += r#"<img />"#,
            }

            elements += &format!(
                "<div>{} - {}</div>",
                file.mime,
                format_size(file.size, DECIMAL),
            );

            file_elements += &format!(r#"<div class="file">{elements}</div>"#);
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
