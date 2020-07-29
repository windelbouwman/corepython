/// Location in the source file.
#[derive(Clone, Debug, Default)]
pub struct Location {
    pub row: usize,
    pub column: usize,
}

impl Location {
    pub fn get_text_for_user(&self) -> String {
        format!("{}:{}", self.row, self.column)
    }
}
