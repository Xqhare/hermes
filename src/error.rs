use nabu::XffValue;

pub type Result<T> = std::result::Result<T, HermesError>;

#[derive(Debug)]
pub enum HermesError {
    PathError(PathError),
    IOError(std::io::Error),
    NabuError(String),
    TimeOutError,
    ServerError(XffValue),
}

impl HermesError {
    pub fn get_inner_io_error(&self) -> Option<&std::io::Error> {
        match self {
            HermesError::IOError(e) => Some(e),
            _ => None,
        }
    }
    pub fn get_inner_nabu_error(&self) -> Option<String> {
        match self {
            HermesError::NabuError(e) => Some(e.clone()),
            _ => None,
        }
    }
    pub fn get_inner_server_error(&self) -> Option<XffValue> {
        match self {
            HermesError::ServerError(e) => Some(e.clone()),
            _ => None,
        }
    }
    pub fn get_inner_path_error(&self) -> Option<&PathError> {
        match self {
            HermesError::PathError(e) => Some(e),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum PathError {
    NotDirectory,
    EmptyPath,
}
