//! Compress outgoing HTTP

#![warn(missing_docs)]

use std::cell::RefCell;
use std::fmt;
use std::io::prelude::*;

use afire::{
    middleware::{MiddleResponse, Middleware},
    Content, Header, Request, Response, Server,
};

use libflate::deflate;

/// Compression Methods
#[derive(Debug, Clone, Copy)]
pub enum CompressType {
    /// Gzip Compression
    ///
    /// The number is the quality (0-9)
    Gzip(u32),

    /// Deflate Compression
    Deflate,

    /// Brotli Compression
    ///
    /// The number is the quality (0-9)
    Brotli(u32),
}

/// Compression Middleware
#[derive(Debug, Clone, Copy)]
pub struct Compress {
    compression: CompressType,
    threshold: usize,
}

impl Middleware for Compress {
    fn post(&mut self, req: Request, res: Response) -> MiddleResponse {
        // Don't compress if body is under threshold
        if res.data.len() <= self.threshold {
            return MiddleResponse::Continue;
        }

        // Check if client doesn't support compression
        match req.header("Accept-Encoding") {
            Some(i) => {
                if !i
                    .split(',')
                    .map(|x| x.trim().to_owned())
                    .collect::<Vec<_>>()
                    .contains(&self.compression.to_string())
                {
                    return MiddleResponse::Continue;
                }
            }
            None => return MiddleResponse::Continue,
        }

        // Compress with specified method
        let new = match self.compression {
            CompressType::Gzip(level) => {
                let mut encoder =
                    flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::new(level));
                encoder.write_all(&res.data).unwrap();
                (encoder.finish().unwrap(), "gzip")
            }

            CompressType::Deflate => {
                let mut encoder = deflate::Encoder::new(Vec::new());
                encoder.write_all(&res.data).unwrap();
                (encoder.finish().into_result().unwrap(), "deflate")
            }

            CompressType::Brotli(level) => {
                let encoder = brotli2::read::BrotliEncoder::new(&*res.data, level);
                (encoder.into_inner().to_vec(), "br")
            }
        };

        MiddleResponse::Add(
            res.bytes(new.0)
                .header(Header::new("Content-Encoding", new.1))
                .content(Content::TXT),
        )
    }
}

impl Compress {
    /// Make a new Compressor
    pub fn new() -> Self {
        Compress {
            compression: CompressType::Gzip(6),
            threshold: 1024,
        }
    }

    /// Set the body size threshold.
    /// This stops from compressing tiny amounts of data
    ///
    /// Default is 1024
    pub fn threshold(self, threshold: usize) -> Self {
        Compress { threshold, ..self }
    }

    /// Set compression method
    ///
    /// Default is Gzip(6)
    pub fn compression(self, compression: CompressType) -> Self {
        Compress {
            compression,
            ..self
        }
    }

    /// Attach Compressor to a server
    pub fn attach(self, server: &mut Server)
    where
        Self: Sized + 'static,
    {
        server.middleware.push(Box::new(RefCell::new(self)));
    }
}

impl fmt::Display for CompressType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                CompressType::Gzip(_) => "gzip",
                CompressType::Deflate => "deflate",
                CompressType::Brotli(_) => "br",
            }
        )
    }
}

impl Default for Compress {
    fn default() -> Self {
        Self::new()
    }
}
