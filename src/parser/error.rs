pub type ParseResult<T> = Result<T, Error>;

#[derive(Debug)]
pub struct Error {
    pub index: usize,
    pub delta: usize, // TODO: error location!
    pub kind: ErrorKind,
}

impl Error {
    pub(crate) fn new(kind: ErrorKind) -> Self {
        Self {
            index: 0,
            delta: 0,
            kind,
        }
    }

    pub(crate) fn raise<T>(kind: ErrorKind) -> Result<T, Self> {
        Err(Self::new(kind))
    }
}

#[cfg(feature = "std")]
impl From<::std::io::Error> for Error {
    fn from(value: ::std::io::Error) -> Self {
        Error::new(ErrorKind::Io(value))
    }
}

#[derive(Debug)]
pub enum ErrorKind {
    #[cfg(feature = "std")]
    Io(::std::io::Error),
    InvalidUtf8,
    Mismatched,
}
