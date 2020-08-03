use super::parser::Location;

#[derive(Debug)]
pub struct CompilationError {
    pub location: Option<Location>,
    pub message: String,
}

impl CompilationError {
    pub fn new(location: &Location, message: &str) -> Self {
        CompilationError {
            location: Some(location.clone()),
            message: message.to_owned(),
        }
    }
}
