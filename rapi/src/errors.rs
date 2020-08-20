use crate::multiplexer;
use reqwest;
use std::error;
use std::fmt;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Io(std::io::Error),
    TMQ(tmq::TmqError),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Reqwest(ref e) => e.fmt(f),
            Error::Io(ref e) => e.fmt(f),
            Error::TMQ(ref e) => e.fmt(f),
        }
    }
}

impl error::Error for Error {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            Error::Reqwest(ref e) => Some(e),
            Error::Io(ref e) => Some(e),
            Error::TMQ(ref e) => Some(e),
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Error {
        Error::Reqwest(err)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Error {
        Error::Io(err)
    }
}

impl From<tmq::TmqError> for Error {
    fn from(err: tmq::TmqError) -> Error {
        Error::TMQ(err)
    }
}

#[derive(Debug)]
pub enum FetchError {
    RecvError,
    SendError(mpsc::error::SendError<multiplexer::Job>),
    DecodeError(prost::DecodeError),
    EncodeError(prost::EncodeError),
}

impl fmt::Display for FetchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "internal error")
    }
}

impl From<prost::DecodeError> for FetchError {
    fn from(err: prost::DecodeError) -> FetchError {
        FetchError::DecodeError(err)
    }
}

impl From<prost::EncodeError> for FetchError {
    fn from(err: prost::EncodeError) -> FetchError {
        FetchError::EncodeError(err)
    }
}

impl From<mpsc::error::SendError<multiplexer::Job>> for FetchError {
    fn from(err: mpsc::error::SendError<multiplexer::Job>) -> FetchError {
        FetchError::SendError(err)
    }
}
