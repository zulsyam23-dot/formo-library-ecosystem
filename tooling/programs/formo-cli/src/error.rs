#[derive(Debug)]
pub struct CliError {
    pub message: String,
    pub already_printed: bool,
}

impl CliError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            already_printed: false,
        }
    }

    pub fn printed(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            already_printed: true,
        }
    }
}

impl From<String> for CliError {
    fn from(value: String) -> Self {
        CliError::new(value)
    }
}

impl From<&str> for CliError {
    fn from(value: &str) -> Self {
        CliError::new(value)
    }
}
