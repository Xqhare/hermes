use std::path::Path;

use crate::{error, status_signal::get_all_status_signals, ERROR_FILE};

pub fn delete_any_status_set(path: &Path) -> Result<(), std::io::Error> {
    for status in get_all_status_signals() {
        if path.join(status.to_string()).exists() {
            std::fs::remove_file(path.join(status.to_string()))?;
        }
    }
    Ok(())
}

pub fn prepare_directory(path: &Path) -> Result<(), error::HermesError> {
    if !path.exists() {
        // create directory
        if let Err(e) = std::fs::create_dir_all(&path) {
            return Err(error::HermesError::IOError(e));
        }
    } else {
        // handle existing directory - no status or error files
        if let Err(e) = delete_any_status_set(path) {
            return Err(error::HermesError::IOError(e));
        }
        if path.join(ERROR_FILE).exists() {
            if let Err(e) = std::fs::remove_file(path.join(ERROR_FILE)) {
                return Err(error::HermesError::IOError(e));
            }
        }
        // If `request.xff` or `response.xff` exist, it doesn't matter, as they should
        // never be read without the status-signal file
    }
    Ok(())
}
