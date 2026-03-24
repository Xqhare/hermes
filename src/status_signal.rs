use std::fmt::Display;
use std::path::Path;

pub fn delete_any_status_set(path: &Path) -> Result<(), std::io::Error> {
    for status in get_all_status_signals() {
        if path.join(status.to_string()).exists() {
            std::fs::remove_file(path.join(status.to_string()))?;
        }
    }
    Ok(())
}

pub fn get_all_status_signals() -> Vec<StatusSignal> {
    vec![
        StatusSignal::StatusOpen,
        StatusSignal::StatusDone,
        StatusSignal::StatusError,
    ]
}

pub enum StatusSignal {
    StatusOpen,
    StatusDone,
    StatusError,
}

impl Display for StatusSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StatusSignal::StatusOpen => write!(f, "status-open"),
            StatusSignal::StatusDone => write!(f, "status-done"),
            StatusSignal::StatusError => write!(f, "status-error"),
        }
    }
}
