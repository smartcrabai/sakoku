use std::fmt;
use std::io;

#[derive(Debug)]
pub enum SakokuError {
    Io(io::Error),
    Walk(ignore::Error),
}

impl fmt::Display for SakokuError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(e) => write!(f, "I/O error: {e}"),
            Self::Walk(e) => write!(f, "walk error: {e}"),
        }
    }
}

impl From<io::Error> for SakokuError {
    fn from(e: io::Error) -> Self {
        Self::Io(e)
    }
}

impl From<ignore::Error> for SakokuError {
    fn from(e: ignore::Error) -> Self {
        Self::Walk(e)
    }
}
