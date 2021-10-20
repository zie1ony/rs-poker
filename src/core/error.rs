use thiserror::Error;

#[derive(Error, Debug)]
pub enum RSPokerError {
    #[error("Unable to parse value")]
    UnexpectedValueChar,
    #[error("Unable to parse suit")]
    UnexpectedSuitChar,
    #[error("Error reading characters while parsing card")]
    TooFewChars,
}
