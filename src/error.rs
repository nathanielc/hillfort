use diesel::result::Error as DError;
use rocket::http::Status;
use std::io;
use std::{fmt, string::FromUtf8Error};
use std::{num::ParseIntError, str::Utf8Error};

#[derive(Debug, Clone, PartialEq)]
pub enum Code {
    Internal,
    Invalid,
    NotFound,
}
#[derive(Debug, Clone)]
pub struct Error {
    pub code: Code,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.code)
    }
}

impl Error {
    pub fn is_not_found(&self) -> bool {
        self.code == Code::NotFound
    }
}

impl From<Utf8Error> for Error {
    fn from(_: Utf8Error) -> Self {
        Error {
            code: Code::Invalid,
        }
    }
}
impl From<FromUtf8Error> for Error {
    fn from(_: FromUtf8Error) -> Self {
        Error {
            code: Code::Internal,
        }
    }
}
impl From<ParseIntError> for Error {
    fn from(_: ParseIntError) -> Self {
        Error {
            code: Code::Invalid,
        }
    }
}
impl From<DError> for Error {
    fn from(e: DError) -> Self {
        match e {
            DError::NotFound => Error {
                code: Code::NotFound,
            },
            _ => Error {
                code: Code::Internal,
            },
        }
    }
}
impl From<io::Error> for Error {
    fn from(_: io::Error) -> Self {
        Error {
            code: Code::Internal,
        }
    }
}

impl From<Error> for Status {
    fn from(e: Error) -> Self {
        match e.code {
            Code::Internal => Status::InternalServerError,
            Code::Invalid => Status::BadRequest,
            Code::NotFound => Status::NotFound,
        }
    }
}
