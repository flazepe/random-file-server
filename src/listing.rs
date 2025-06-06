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
        let total_pages = (total_paths as f64 / FILES_PER_PAGE as f64).round() as usize;
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
            let mime_type = from_path(path).first_or_octet_stream();

            let mut elements = format!(
                r#"<a href="{}" target="_blank">{}</a>"#,
                path.to_string_lossy(),
                encode_text(
                    path.to_string_lossy()
                        .split('/')
                        .next_back()
                        .unwrap_or_default(),
                ),
            );

            match mime_type
                .essence_str()
                .split('/')
                .next()
                .unwrap_or_default()
            {
                tag @ ("audio" | "video") => {
                    elements += &format!(
                        r#"<{tag} src="{}" controls></{tag}>"#,
                        path.to_string_lossy(),
                    )
                }
                "image" => elements += &format!(r#"<img src="{}" />"#, path.to_string_lossy()),
                _ => elements += r#"<img />"#,
            }

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

        let html = format!(
            r#"
			<!DOCTYPE html>
			<html lang="en">
                <head>
                    <title>File Listing</title>
                    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                    <style>
                        body {{
                            font-family: Segoe UI, Arial, Helvetica, sans-serif;
                        }}

                        .container {{
                            display: flex;
                            flex-direction: column;
                            align-items: center;
                            justify-items: center;
                            gap: 20px;
                            margin: 20px;
                        }}

                        .paginator {{
                            display: flex;
                            flex-wrap: wrap;
                            gap: 10px;
                            justify-content: center;
                            border-radius: 10px;
                            background-color: #f1f1f1;
                            padding: 10px;

                            a {{
                                display: flex;
                                align-items: center;
                                justify-content: center;
                                text-decoration: none;
                                padding: 5px;
                                border-radius: 50%;
                                width: 20px;
                                height: 20px;
                            }}

                            a {{
                                background-color: #ccc;
                                color: #000;
                            }}

                            a.current {{
                                background-color: #3170bd;
                                color: #fff;
                            }}
                        }}

                        .files {{
                            display: flex;
                            flex-direction: column;
                            align-items: center;
                            justify-content: center;
                            gap: 20px;
                        }}

                        @media only screen and (min-width: 512px) {{
                            .files {{
                                flex-direction: row;
                                flex-wrap: wrap;
                            }}
                        }}

                        .file {{
                            display: flex;
                            flex-direction: column;
                            align-items: center;
                            gap: 20px;
                            word-break: break-all;

                            a {{
                                text-decoration: none;
                            }}

                            img, video {{
                                width: 80%;
                            }}
                        }}

                        @media only screen and (min-width: 512px) {{
                            .file {{
                                img, video {{
                                    width: 300px;
                                }}
                            }}
                        }}
                    </style>
                </head>
                <body>
                <div class="container">
                    {info_element}
                    <div class="paginator">{paginator_elements}</div>
                    <div class="files">{file_elements}</div>
                    <div class="paginator">{paginator_elements}</div>
                </div>
                </body>
			</html>
			"#
        );

        let mut response = Response::from_string(html);

        if let Ok(header) = Header::from_bytes("content-type", "text/html") {
            response = response.with_header(header);
        }

        response
    }
}
