use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use bytes::Bytes;
use http::uri::Uri;
use hyper::{Body, Client};
use hyper_rustls::HttpsConnectorBuilder;
use tokio::fs::read;

/// A path of a resource.
#[derive(Debug, Clone)]
pub enum Path {
    /// A remote resource should be fetched via newwork.
    Url(Uri),

    /// A local resource.
    PathBuf(PathBuf),
}

pub fn parse_string_to_path(s: String) -> Result<Path> {
    if s.starts_with('.') {
        return Ok(Path::PathBuf(PathBuf::from(s)));
    }

    let uri = s.parse::<Uri>()?;

    match uri.scheme_str() {
        Some("http") | Some("https") => Ok(Path::Url(uri)),

        None => Ok(Path::PathBuf(PathBuf::from(s))),

        _ => Err(anyhow!("Unknown scheme in `{}`", s)),
    }
}

pub async fn load_content_from_url(path: Path) -> Result<Bytes> {
    match path {
        Path::Url(url) => match url.scheme_str() {
            Some("http") => {
                let client = Client::new();
                let resource_string = format!("remote resource `{url}`");
                let resp = client
                    .get(url)
                    .await
                    .with_context(|| format!("failed to fetch {resource_string}"))?;
                Ok(hyper::body::to_bytes(resp.into_body())
                    .await
                    .with_context(|| {
                        format!("failed to convert response body to bytes in {resource_string}")
                    })?)
            }

            Some("https") => {
                let https = HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_only()
                    .enable_http1()
                    .enable_http2()
                    .build();
                let client: Client<_, Body> = Client::builder().build(https);
                let resource_string = format!("remote resource `{url}`");
                let resp = client
                    .get(url)
                    .await
                    .with_context(|| format!("failed to fetch {resource_string}"))?;
                Ok(hyper::body::to_bytes(resp.into_body())
                    .await
                    .with_context(|| {
                        format!("failed to convert response body to bytes in {resource_string}")
                    })?)
            }

            _ => unreachable!(),
        },

        Path::PathBuf(path_buf) => {
            let read_err_msg = format!("failed to read local file `{}`", path_buf.display());
            let contents = read(path_buf).await.context(read_err_msg)?;
            Ok(Bytes::from(contents))
        }
    }
}
