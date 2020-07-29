use super::parser::Location;

pub struct CompilationError {
    pub location: Option<Location>,
    pub message: String,
}
