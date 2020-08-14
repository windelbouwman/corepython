use super::parser::Location;

#[derive(Debug)]
pub struct CompilationError {
    pub location: Option<Location>,
    pub message: String,
}

impl CompilationError {
    pub fn new<S: Into<String>>(location: &Location, message: S) -> Self {
        CompilationError {
            location: Some(location.clone()),
            message: message.into(),
        }
    }
}
