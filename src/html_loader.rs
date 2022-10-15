//! Provides a html loader with compression support
use reqwest::header::{
    ACCEPT, ACCEPT_ENCODING, ACCEPT_LANGUAGE, CONTENT_ENCODING, USER_AGENT,
};
use std::io::Read;

use crate::FletcherError;

/// Tries to download a single URL 10 times and returns the body
pub fn load_page(url: &str) -> Result<String, FletcherError> {
    for _ in 1..=10 {
        let client = reqwest::blocking::Client::new();
        let mut res = match client
            .get(url)
            .header(ACCEPT_ENCODING, "gzip, deflate, br")
            .header(ACCEPT_LANGUAGE, "de,en-US;q=0.7,en;q=0.3")
            .header(ACCEPT, "text/html,application/xhtml+xml,application/xml;q=0.9,image/avif,image/webp,*/*;q=0.8")
            .header(USER_AGENT, "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/100.0.4896.127 Safari/537.36")
            .send()
        {
            Ok(response) => response,
            Err(_) => {
                continue;
            }
        };

        let mut body = String::new();

        if res.headers().contains_key(CONTENT_ENCODING) {
            let mut buf: Vec<u8> = Vec::new();
            match res.read_to_end(&mut buf) {
                Ok(_) => {}
                Err(_) => {
                    continue;
                }
            }
            match res
                .headers()
                .get(CONTENT_ENCODING)
                .unwrap()
                .to_str()
                .unwrap()
            {
                "gzip" => {
                    let mut decoder =
                        libflate::gzip::Decoder::new(&buf[..]).unwrap();
                    if decoder.read_to_string(&mut body).is_err() {
                        continue;
                    }
                }
                "deflate" => {
                    let mut decoder = libflate::deflate::Decoder::new(&buf[..]);
                    if decoder.read_to_string(&mut body).is_err() {
                        continue;
                    }
                }
                "br" => {
                    let mut decoder =
                        brotli::Decompressor::new(&buf[..], buf.len());
                    if decoder.read_to_string(&mut body).is_err() {
                        continue;
                    }
                }
                _ => return Err(FletcherError::InvalidResponse),
            };
        } else if res.read_to_string(&mut body).is_err() {
            continue;
        }

        if res.status().is_success() {
            return Ok(body);
        }

        if res.status().eq(&reqwest::StatusCode::TOO_MANY_REQUESTS) {
            return Err(FletcherError::RateLimitReached);
        }

        if res.status().eq(&reqwest::StatusCode::FORBIDDEN) {
            return Err(FletcherError::Forbidden);
        }

        if res.status().eq(&reqwest::StatusCode::NOT_FOUND) {
            return Err(FletcherError::NotFound);
        }
    }

    Err(FletcherError::FailedTooOften)
}
