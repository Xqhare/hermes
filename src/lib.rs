use std::path::PathBuf;

use nabu::XffValue;
use error::Result;
use status_signal::{delete_any_status_set, StatusSignal};

pub mod error;
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
    /// assert!(hermes.is_ok());
    /// # assert!(std::fs::remove_dir_all("tmp").is_ok());
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
        if !path.exists() {
            // create directory
            if let Err(e) = std::fs::create_dir_all(&path) {
                return Err(error::HermesError::IOError(e));
            }
        }
        Ok(
            Hermes {
                path, 
                garbage_collection: false 
            }
        )
    }

    pub fn request(&self, value: XffValue) -> Result<()> {
        let (req_file, status_signal_file) = (self.path.join(REQUEST_FILE), self.path.join(StatusSignal::StatusOpen.to_string()));
        if let Err(e) = nabu::serde::write(req_file, value) {
            return Err(error::HermesError::NabuError(e.to_string()));
        }
        if let Err(e) = std::fs::File::create(status_signal_file) {
            return Err(error::HermesError::IOError(e));
        }
        Ok(())
    }

    fn await_req(&self, max_iterations: u32, break_on_max: bool) -> Result<XffValue> {
        let signal_status_file = self.path.join(StatusSignal::StatusOpen.to_string());
        let mut count = 0;
        let error_file = self.path.join(ERROR_FILE);
        while !signal_status_file.exists() && !error_file.exists() {
            // This takes with a max_iter of 100_000 about 72s
            // and with a max_iter of 150_000 about 197s
            if count > 100_000 {
                std::thread::sleep(std::time::Duration::from_micros(2_500));
            } else if count > 90_000 {
                std::thread::sleep(std::time::Duration::from_micros(1_250));
            } else if count > 75_000 {
                std::thread::sleep(std::time::Duration::from_micros(1_000));
            } else if count > 50_000 {
                std::thread::sleep(std::time::Duration::from_micros(750));
            } else if count > 25_000 {
                std::thread::sleep(std::time::Duration::from_micros(500));
            } else if count > 10_000 {
                std::thread::sleep(std::time::Duration::from_micros(250));
            } else if count > 5_000 {
                std::thread::sleep(std::time::Duration::from_micros(200));
            } else if count > 2_500 {
                std::thread::sleep(std::time::Duration::from_micros(150));
            } else {
                // This takes about 0.25s
                std::thread::sleep(std::time::Duration::from_micros(100));
            }
            count += 1;
            if count > max_iterations && break_on_max {
                return Err(error::HermesError::TimeOutError);
            }
        }
        if error_file.exists() {
            let error = nabu::serde::read(error_file);
            if error.is_err() {
                return Err(error::HermesError::NabuError(error.unwrap_err().to_string()));
            }
            return Err(error::HermesError::ServerError(error.unwrap()));
        }
        let answer = nabu::serde::read(self.path.join(REQUEST_FILE));
        if answer.is_err() {
            return Err(error::HermesError::NabuError(answer.unwrap_err().to_string()));
        }
        if let Err(e) = std::fs::remove_file(signal_status_file) {
            return Err(error::HermesError::IOError(e));
        }
        if self.garbage_collection {
            if let Err(e) = self.free_resources() {
                return Err(e);
            }
        }
        Ok(answer.unwrap())
    }

    /// Waits for a request, without a timeout, literally forever
    pub fn await_request_forever(&self) -> Result<XffValue> {
        let break_on_max = false;
        self.await_req(0, break_on_max)
    }

    /// Waits for a request, with a timeout of around 200 seconds
    pub fn await_request(&self) -> Result<XffValue> {
        let break_on_max = true;
        self.await_req(150_000, break_on_max)
    }

    pub fn respond(&self, value: XffValue) -> Result<()> {
        let (res_file, status_signal_file) = (self.path.join(RESPONSE_FILE), self.path.join(StatusSignal::StatusDone.to_string()));
        if let Err(e) = nabu::serde::write(res_file, value) {
            return Err(error::HermesError::NabuError(e.to_string()));
        }
        if let Err(e) = std::fs::File::create(status_signal_file) {
            return Err(error::HermesError::IOError(e));
        }
        Ok(())
    }

    fn await_resp(&self, max_iterations: u32, break_on_max: bool) -> Result<XffValue> {
        let signal_status_file = self.path.join(StatusSignal::StatusDone.to_string());
        let mut count = 0;
        let error_file = self.path.join(ERROR_FILE);
        while !signal_status_file.exists() && !error_file.exists() {
            // This takes with a max_iter of 100_000 about 72s
            // and with a max_iter of 150_000 about 197s
            if count > 100_000 {
                std::thread::sleep(std::time::Duration::from_micros(2_500));
            } else if count > 90_000 {
                std::thread::sleep(std::time::Duration::from_micros(1_250));
            } else if count > 75_000 {
                std::thread::sleep(std::time::Duration::from_micros(1_000));
            } else if count > 50_000 {
                std::thread::sleep(std::time::Duration::from_micros(750));
            } else if count > 25_000 {
                std::thread::sleep(std::time::Duration::from_micros(500));
            } else if count > 10_000 {
                std::thread::sleep(std::time::Duration::from_micros(250));
            } else if count > 5_000 {
                std::thread::sleep(std::time::Duration::from_micros(200));
            } else if count > 2_500 {
                std::thread::sleep(std::time::Duration::from_micros(150));
            } else {
                // This takes about 0.25s
                std::thread::sleep(std::time::Duration::from_micros(100));
            }
            count += 1;
            if count > max_iterations && break_on_max {
                return Err(error::HermesError::TimeOutError);
            }
        }
        if error_file.exists() {
            let error = nabu::serde::read(error_file);
            if error.is_err() {
                return Err(error::HermesError::NabuError(error.unwrap_err().to_string()));
            }
            return Err(error::HermesError::ServerError(error.unwrap()));
        }
        let answer = nabu::serde::read(self.path.join(RESPONSE_FILE));
        if answer.is_err() {
            return Err(error::HermesError::NabuError(answer.unwrap_err().to_string()));
        }
        if let Err(e) = std::fs::remove_file(signal_status_file) {
            return Err(error::HermesError::IOError(e));
        }
        if self.garbage_collection {
            if let Err(e) = self.free_resources() {
                return Err(e);
            }
        }
        Ok(answer.unwrap())
    }

    pub fn await_response(&self) -> Result<XffValue> {
        let break_on_max = true;
        self.await_resp(150_000, break_on_max)
    }

    pub fn await_response_forever(&self) -> Result<XffValue> {
        let break_on_max = false;
        self.await_resp(0, break_on_max)
    }

    /// Frees disk resources used by Hermes.
    pub fn free_resources(&self) -> Result<()> {
        let (req_file, resp_file, err_file) = (self.path.join(REQUEST_FILE), self.path.join(RESPONSE_FILE), self.path.join(ERROR_FILE));
        if let Err(e) = std::fs::remove_file(req_file) {
            return Err(error::HermesError::IOError(e));
        }
        if let Err(e) = std::fs::remove_file(resp_file) {
            return Err(error::HermesError::IOError(e));
        }
        if let Err(e) = std::fs::remove_file(err_file) {
            return Err(error::HermesError::IOError(e));
        }
        Ok(())
    }

    /// Sets the garbage collection flag.
    /// If set to true, Hermes will free disk resources used by Hermes. This will result in
    /// more disk activity.
    ///
    /// Consider using `free_resources` instead if the increased disk activity is a problem.
    pub fn set_garbage_collection(&mut self, enabled: bool) {
        self.garbage_collection = enabled;
    }

    /// Puts an error in the error file. Used to signal a server error and send it to the
    /// Client.
    ///
    /// Should this also throw an Error it's probably time to panic!
    pub fn put_error(&self, error: XffValue) -> Result<()> {
        if let Err(e) = delete_any_status_set(&self.path) {
            return Err(error::HermesError::IOError(e));
        }
        let error_file = self.path.join(ERROR_FILE);
        let status_signal_file = self.path.join(StatusSignal::StatusError.to_string());
        if let Err(e) = nabu::serde::write(error_file, error) {
            return Err(error::HermesError::NabuError(e.to_string()));
        }
        if let Err(e) = std::fs::File::create(status_signal_file) {
            return Err(error::HermesError::IOError(e));
        }
        Ok(())
    }
}
