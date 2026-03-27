#![doc = include_str!("../README.md")]

use std::path::PathBuf;

use error::Result;
use nabu::XffValue;
use status_signal::{StatusSignal, delete_any_status_set};

pub mod error;
mod status_signal;

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
    /// let hermes = Hermes::new("folder/hermes");
    /// assert!(hermes.is_ok());
    /// # assert!(std::fs::remove_dir_all("folder").is_ok());
    ///
    /// let hermes = Hermes::new("hermes");
    /// assert!(hermes.is_ok());
    /// # assert!(std::fs::remove_dir_all("hermes").is_ok());
    ///
    /// let hermes = Hermes::new("");
    /// assert!(hermes.is_err());
    /// ```
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Hermes> {
        let path = path.into();
        if path.to_string_lossy().is_empty() {
            return Err(error::HermesError::PathError(error::PathError::EmptyPath));
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
        Ok(Hermes {
            path,
            garbage_collection: false,
        })
    }

    /// Sends a request to the server.
    ///
    /// # Arguments
    /// * `value` - The value you want to send to Hermes.
    ///
    /// # Errors
    /// If system calls fail, returns an error.
    ///
    /// # Examples
    /// ```
    /// # use hermes::Hermes;
    /// # use nabu::XffValue;
    /// let hermes = Hermes::new("req").unwrap();
    /// let value: XffValue = "".into();
    /// assert!(hermes.request(value).is_ok());
    /// # assert!(std::fs::remove_dir_all("req").is_ok());
    /// ```
    pub fn request(&self, value: XffValue) -> Result<()> {
        let (req_file, status_signal_file) = (
            self.path.join(REQUEST_FILE),
            self.path.join(StatusSignal::StatusOpen.to_string()),
        );
        if let Err(e) = nabu::serde::write(req_file, value) {
            return Err(error::HermesError::NabuError(e.to_string()));
        }
        if let Err(e) = std::fs::File::create(status_signal_file) {
            return Err(error::HermesError::IOError(e));
        }
        Ok(())
    }

    /// Checks if a request is ready.
    /// Use with `get_request` to implement your own Serverside `await`
    ///
    /// # Examples
    /// ```
    /// # use hermes::Hermes;
    /// # use nabu::XffValue;
    /// let hermes = Hermes::new("req_ready").unwrap();
    /// let value: XffValue = "".into();
    /// assert!(hermes.request(value).is_ok());
    /// assert!(hermes.is_request_ready());
    /// # assert!(std::fs::remove_dir_all("req_ready").is_ok());
    /// ```
    #[must_use] 
    pub fn is_request_ready(&self) -> bool {
        self.path
            .join(StatusSignal::StatusOpen.to_string())
            .exists()
    }

    /// Attempts to immideatly get the request from Hermes.
    ///
    /// # Errors
    /// If system calls fail or the request does not exist, returns an error.
    ///
    /// # Examples
    /// ```
    /// # use hermes::Hermes;
    /// # use nabu::XffValue;
    /// let hermes = Hermes::new("get_req").unwrap();
    /// let value: XffValue = "".into();
    /// assert!(hermes.request(value).is_ok());
    /// assert!(hermes.is_request_ready());
    /// let request = hermes.get_request();
    /// assert!(request.is_ok());
    /// assert_eq!(request.unwrap(), "".into());
    /// # assert!(std::fs::remove_dir_all("get_req").is_ok());
    /// ```
    pub fn get_request(&self) -> Result<XffValue> {
        let req_file = self.path.join(REQUEST_FILE);
        if !req_file.exists() {
            return Err(error::HermesError::RequestError(
                error::RequestError::RequestDoesNotExist,
            ));
        }
        let request = nabu::serde::read(req_file);
        if let Err(e) = request {
            return Err(error::HermesError::NabuError(e.to_string()));
        }
        Ok(request.unwrap())
    }

    /// Internal await function
    ///
    /// # Arguments
    /// * `max_iterations` - The maximum number of iterations to wait for a response.
    /// * `break_on_max` - If true, will return an error if the maximum number of iterations is reached.
    ///
    /// # Errors
    /// If system calls fail, if the maximum number of iterations is reached and `break_on_max` is true, or if an error file exists, returns an error.
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
                return Err(error::HermesError::NabuError(
                    error.unwrap_err().to_string(),
                ));
            }
            return Err(error::HermesError::ServerError(error.unwrap()));
        }
        let answer = nabu::serde::read(self.path.join(REQUEST_FILE));
        if answer.is_err() {
            return Err(error::HermesError::NabuError(
                answer.unwrap_err().to_string(),
            ));
        }
        if let Err(e) = std::fs::remove_file(signal_status_file) {
            return Err(error::HermesError::IOError(e));
        }
        if self.garbage_collection {
            self.free_resources()?;
        }
        Ok(answer.unwrap())
    }

    /// Waits for a request, without a timeout, literally forever
    ///
    /// # Errors
    /// If system calls fail, returns an error
    ///
    /// # Examples
    /// ```
    /// # use hermes::Hermes;
    /// # use nabu::XffValue;
    /// let hermes = Hermes::new("await_req_forever").unwrap();
    /// let value: XffValue = "".into();
    /// assert!(hermes.request(value).is_ok());
    /// assert!(hermes.is_request_ready());
    /// let request = hermes.await_request_forever();
    /// assert!(request.is_ok());
    /// assert_eq!(request.unwrap(), "".into());
    /// # assert!(std::fs::remove_dir_all("await_req_forever").is_ok());
    /// ```
    pub fn await_request_forever(&self) -> Result<XffValue> {
        let break_on_max = false;
        self.await_req(0, break_on_max)
    }

    /// Waits for a request, with a timeout of around 3 minutes
    ///
    /// # Errors
    /// If system calls fail, returns an error
    ///
    /// # Examples
    /// ```
    /// # use hermes::Hermes;
    /// # use nabu::XffValue;
    /// let hermes = Hermes::new("await_req").unwrap();
    /// let value: XffValue = "".into();
    /// assert!(hermes.request(value).is_ok());
    /// assert!(hermes.is_request_ready());
    /// let request = hermes.await_request();
    /// assert!(request.is_ok());
    /// assert_eq!(request.unwrap(), "".into());
    /// # assert!(std::fs::remove_dir_all("await_req").is_ok());
    /// ```
    pub fn await_request(&self) -> Result<XffValue> {
        let break_on_max = true;
        self.await_req(150_000, break_on_max)
    }

    /// Responds to a request from the client
    ///
    /// # Arguments
    /// * `value` - The value you want to send to Hermes
    ///
    /// # Errors
    /// If system calls fail, returns an error
    ///
    /// # Examples
    /// ```
    /// # use hermes::Hermes;
    /// # use nabu::XffValue;
    /// let hermes = Hermes::new("res").unwrap();
    /// let value: XffValue = "".into();
    /// assert!(hermes.respond(value).is_ok());
    /// # assert!(std::fs::remove_dir_all("res").is_ok());
    /// ```
    pub fn respond(&self, value: XffValue) -> Result<()> {
        let (res_file, status_signal_file) = (
            self.path.join(RESPONSE_FILE),
            self.path.join(StatusSignal::StatusDone.to_string()),
        );
        if let Err(e) = nabu::serde::write(res_file, value) {
            return Err(error::HermesError::NabuError(e.to_string()));
        }
        if let Err(e) = std::fs::File::create(status_signal_file) {
            return Err(error::HermesError::IOError(e));
        }
        Ok(())
    }

    /// Used for manual polling of the status of a request.
    /// Use with `get_request` to implement your own `await`
    ///
    /// # Examples
    /// ```
    /// # use hermes::Hermes;
    /// # use nabu::XffValue;
    /// let hermes = Hermes::new("res_ready").unwrap();
    /// let value: XffValue = "".into();
    /// assert!(hermes.respond(value).is_ok());
    /// assert!(hermes.is_response_ready());
    /// # assert!(std::fs::remove_dir_all("res_ready").is_ok());
    /// ```
    #[must_use] 
    pub fn is_response_ready(&self) -> bool {
        self.path
            .join(StatusSignal::StatusDone.to_string())
            .exists()
    }

    /// Attempts to immideatly get the response from Hermes.
    ///
    /// # Errors
    /// If system calls fail or the response does not exist, returns an error.
    ///
    /// # Examples
    /// ```
    /// # use hermes::Hermes;
    /// # use nabu::XffValue;
    /// let hermes = Hermes::new("get_res").unwrap();
    /// let value: XffValue = "".into();
    /// assert!(hermes.respond(value).is_ok());
    /// assert!(hermes.is_response_ready());
    /// let response = hermes.get_response();
    /// println!("{:?}", response);
    /// assert!(response.is_ok());
    /// assert_eq!(response.unwrap(), "".into());
    /// # assert!(std::fs::remove_dir_all("get_res").is_ok());
    /// ```
    pub fn get_response(&self) -> Result<XffValue> {
        let req_file = self.path.join(RESPONSE_FILE);
        if !req_file.exists() {
            return Err(error::HermesError::ResponseError(
                error::ResponseError::ResponseDoesNotExist,
            ));
        }
        let answer = nabu::serde::read(req_file);
        if let Err(e) = answer {
            return Err(error::HermesError::NabuError(e.to_string()));
        }
        Ok(answer.unwrap())
    }

    /// Internal await function
    ///
    /// # Arguments
    /// * `max_iterations` - The maximum number of iterations to wait for a response.
    /// * `break_on_max` - If true, will return an error if the maximum number of iterations is reached.
    ///
    /// # Errors
    /// If system calls fail, if the maximum number of iterations is reached and `break_on_max` is true, or if an error file exists, returns an error.
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
                return Err(error::HermesError::NabuError(
                    error.unwrap_err().to_string(),
                ));
            }
            return Err(error::HermesError::ServerError(error.unwrap()));
        }
        let answer = nabu::serde::read(self.path.join(RESPONSE_FILE));
        if answer.is_err() {
            return Err(error::HermesError::NabuError(
                answer.unwrap_err().to_string(),
            ));
        }
        if let Err(e) = std::fs::remove_file(signal_status_file) {
            return Err(error::HermesError::IOError(e));
        }
        if self.garbage_collection {
            self.free_resources()?;
        }
        Ok(answer.unwrap())
    }

    /// Waits for a response from Hermes with a timeout of around 3 minutes
    ///
    /// # Errors
    /// If system calls fail, or if an error file exists, returns an error.
    ///
    /// # Examples
    /// ```
    /// # use hermes::Hermes;
    /// # use nabu::XffValue;
    /// let hermes = Hermes::new("await_res").unwrap();
    /// let value: XffValue = "".into();
    /// assert!(hermes.respond(value).is_ok());
    /// let response = hermes.await_response();
    /// assert!(response.is_ok());
    /// assert_eq!(response.unwrap(), "".into());
    /// # assert!(std::fs::remove_dir_all("await_res").is_ok());
    /// ```
    pub fn await_response(&self) -> Result<XffValue> {
        let break_on_max = true;
        self.await_resp(150_000, break_on_max)
    }

    /// Waits for a response from Hermes, without a timeout.
    ///
    /// # Errors
    /// If system calls fail, or if an error file exists, returns an error.
    ///
    /// # Examples
    /// ```
    /// # use hermes::Hermes;
    /// # use nabu::XffValue;
    /// let hermes = Hermes::new("await_res_forever").unwrap();
    /// let value: XffValue = "".into();
    /// assert!(hermes.respond(value).is_ok());
    /// let response = hermes.await_response_forever();
    /// assert!(response.is_ok());
    /// assert_eq!(response.unwrap(), "".into());
    /// # assert!(std::fs::remove_dir_all("await_res_forever").is_ok());
    /// ```
    pub fn await_response_forever(&self) -> Result<XffValue> {
        let break_on_max = false;
        self.await_resp(0, break_on_max)
    }

    /// Frees disk resources used by Hermes.
    ///
    /// Use if the increased disk activity caused by automatic garbage collection is a problem
    /// and the disk space is needed.
    ///
    /// # Errors
    /// If system calls fail, returns an error.
    ///
    /// # Examples
    /// ```
    /// # use hermes::Hermes;
    /// # use nabu::XffValue;
    /// let hermes = Hermes::new("free_res").unwrap();
    /// let value: XffValue = "".into();
    /// assert!(hermes.respond(value).is_ok());
    /// assert!(hermes.is_response_ready());
    /// assert!(hermes.free_resources().is_ok());
    /// # assert!(std::fs::remove_dir_all("free_res").is_ok());
    /// ```
    pub fn free_resources(&self) -> Result<()> {
        let (req_file, resp_file, err_file) = (
            self.path.join(REQUEST_FILE),
            self.path.join(RESPONSE_FILE),
            self.path.join(ERROR_FILE),
        );
        if req_file.exists()
            && let Err(e) = std::fs::remove_file(req_file) {
                return Err(error::HermesError::IOError(e));
            }
        if resp_file.exists()
            && let Err(e) = std::fs::remove_file(resp_file) {
                return Err(error::HermesError::IOError(e));
            }
        if err_file.exists()
            && let Err(e) = std::fs::remove_file(err_file) {
                return Err(error::HermesError::IOError(e));
            }
        Ok(())
    }

    /// Sets the garbage collection flag.
    /// If set to true, Hermes will free disk resources used by Hermes. This will result in
    /// more disk activity.
    ///
    /// Consider using `free_resources` instead if the increased disk activity is a problem.
    ///
    /// # Examples
    /// ```
    /// # use hermes::Hermes;
    /// # use nabu::XffValue;
    /// let mut hermes = Hermes::new("set_gc").unwrap();
    /// hermes.set_garbage_collection(true);
    /// # assert!(std::fs::remove_dir_all("set_gc").is_ok());
    /// ```
    pub fn set_garbage_collection(&mut self, enabled: bool) {
        self.garbage_collection = enabled;
    }

    /// Puts an error in the error file. Used to signal a server error and send it to the
    /// Client.
    ///
    /// Should this also throw an Error it's probably time to panic!
    ///
    /// # Arguments
    /// * `error` - The error you want to send to Hermes - a `XffValue`
    ///
    /// # Errors
    /// If system calls fail, returns an error.
    ///
    /// # Examples
    /// ```
    /// # use hermes::Hermes;
    /// # use nabu::XffValue;
    /// let hermes = Hermes::new("error").unwrap();
    /// let error: XffValue = "".into();
    /// assert!(hermes.put_error(error).is_ok());
    /// assert!(hermes.get_response().is_err());
    /// # assert!(std::fs::remove_dir_all("error").is_ok());
    /// ```
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
