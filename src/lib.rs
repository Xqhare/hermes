use std::path::PathBuf;

use athena::XffValue;
use error::Result;
use utils::prepare_directory;

mod error;
mod status_signal;
mod utils;

const ERROR_FILE: &str = "error.xff";
const REQUEST_FILE: &str = "request.xff";
const RESPONSE_FILE: &str = "response.xff";

#[derive(Debug)]
pub struct Hermes {
    path: PathBuf,
    garbage_collection: bool,
}

impl Hermes {
    /// Creates a new Hermes connection.
    ///
    /// # Arguments
    /// * `path` - The path to the directory you want Hermes to use.
    ///
    /// # Errors
    /// If the path is not a directory, or if the path is empty, returns an error.
    /// Also errors if system calls fail.
    ///
    /// # Examples
    /// ```
    /// use hermes::Hermes;
    /// let hermes = Hermes::new("tmp/hermes");
    /// println!("{:?}", hermes);
    /// assert!(hermes.is_ok());
    /// # std::fs::remove_dir_all("tmp/hermes");
    ///
    /// let hermes = Hermes::new("");
    /// assert!(hermes.is_err());
    /// ```
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Hermes> {
        let path = path.into();
        if path.to_string_lossy().len() == 0 {
            return Err(error::HermesError::PathError(
                error::PathError::EmptyPath,
            ));
        }
        if path.exists() && !path.is_dir() {
            return Err(error::HermesError::PathError(
                error::PathError::NotDirectory,
            ));
        }
        prepare_directory(&path)?;
        Ok(
            Hermes {
                path, 
                garbage_collection: false 
            }
        )
    }

    pub fn request(&self, value: XffValue) {
        todo!()
    }

    pub fn await_request(&self) {
        todo!()
    }

    pub fn respond(&self) {
        todo!()
    }

    pub fn await_response(&self) {
        todo!()
    }

    pub fn free_resources(&self) {
        todo!()
    }

    pub fn set_garbage_collection(&mut self, enabled: bool) {
        self.garbage_collection = enabled;
    }
}
