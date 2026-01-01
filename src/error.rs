
pub type Result<T> = std::result::Result<T, HermesError>;

#[derive(Debug)]
pub enum HermesError {
    PathError(PathError),
    IOError(std::io::Error),
}

#[derive(Debug)]
pub enum PathError {
    NotDirectory,
    EmptyPath,
}
