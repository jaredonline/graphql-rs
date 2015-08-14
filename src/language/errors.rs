#[macro_use]
macro_rules! parse_error {
    ($($arg:tt)*) => (
        Err(ParseError::new(format!($($arg)*)))
    )
}

pub struct ParseError {
    pub description: String
}

impl ParseError {
    pub fn new(msg: String) -> ParseError {
        ParseError {
            description: msg
        }
    }
}
