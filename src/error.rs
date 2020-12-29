use core::fmt;
use std::io;
use std::io::{Cursor, ErrorKind};

use rocket::{Request, Response};
use rocket::http::ContentType;
use rocket::response::{self, Responder};
use serde_json::error::Category;

#[derive(Debug, Clone)]
pub enum Error {
    FileAccess,
    FileContent,
    GenericIO,
    DuplicateReview,
    Json,
}

pub type Result<T> = std::result::Result<T, Error>;

impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        match err.kind() {
            ErrorKind::NotFound | ErrorKind::PermissionDenied | ErrorKind::BrokenPipe => {
                Error::FileAccess
            }
            ErrorKind::InvalidInput | ErrorKind::InvalidData | ErrorKind::UnexpectedEof => {
                Error::FileContent
            }
            _ => {
                Error::GenericIO
            }
        }
    }
}

impl AsRef<[u8]> for Error {
    fn as_ref(&self) -> &[u8] {
        match self {
            Error::FileAccess => {
                "Couldn't read file.".as_bytes()
            }
            Error::FileContent => {
                "Couldn't parse file content.".as_bytes()
            }
            Error::GenericIO => {
                "There was an IO error.".as_bytes()
            }
            Error::DuplicateReview => {
                "This review already exists.".as_bytes()
            }
            Error::Json => {
                "Could't parse JSON.".as_bytes()
            }
        }
    }
}

impl<'r> Responder<'r> for Error {
    fn respond_to(self, _: &Request) -> response::Result<'r> {
        // TODO: Add error page
        Response::build()
            .header(ContentType::Plain)
            .sized_body(Cursor::new(self))
            .ok()
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        match err.classify() {
            Category::Io => {
                Error::FileAccess
            }
            Category::Syntax | Category::Data | Category::Eof => {
                Error::Json
            }
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[...]")
    }
}
