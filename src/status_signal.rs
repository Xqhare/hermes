use std::fmt::Display;

pub fn get_all_status_signals() -> Vec<StatusSignal> {
    vec![StatusSignal::StatusOpen, StatusSignal::StatusDone, StatusSignal::StatusError]
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
