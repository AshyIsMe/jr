use thiserror::Error;

// TODO: https://code.jsoftware.com/wiki/Vocabulary/ErrorMessages
#[derive(Debug, Error)]
pub enum JError {
    #[error("Your assert. line did not produce (a list of all) 1 (true)")]
    AssertionFailure,
    #[error("You interrupted execution with the JBreak icon")]
    Break,
    #[error("While loading script: bad use of if. else. end. etc")]
    ControlError,
    #[error(
        "Invalid valence: The verb doesn't have a definition for the valence it was executed with"
    )]
    // #[error("Invalid value: An argument or operand has an invalid value")] ,
    // #[error("Invalid public assignment: You've used both (z=:) and (z=.) for some name z")] ,
    // #[error("Pun in definitions: A name was referred to as one part of speech, but the definition was later changed to another part of speech")] ,
    DomainError,
    #[error("nonexistent device or file")]
    FileNameError,
    #[error("no file open with that number")]
    FileNumberError,
    #[error("your Fold did not terminate when you expected")]
    FoldLimit,
    #[error("Invalid underscores in a name")]
    IllFormedName,
    #[error("A word starting with a number is not a valid number")]
    IllFormedNumber,
    #[error("accessing out of bounds of your array")]
    IndexError,
    #[error("illegal filename or request")]
    InterfaceError,
    #[error("x and y do not agree, or an argument has invalid length")]
    LengthError,
    #[error("You tried to use an expired locale")]
    LocaleError,
    #[error("number is beyond J's limit")]
    LimitError,
    #[error("result is not a valid number")]
    NaNError,
    #[error("feature not supported yet")]
    NonceError,
    #[error(
        "You attempted an operation on a sparse array that would have required expanding the array"
    )]
    NonUniqueSparseElements,
    #[error("Verbs, and test blocks within explicit definitions, must produce noun results")]
    NounResultWasRequired,
    #[error("string started but not ended")]
    OpenQuote,
    #[error("noun too big for computer")]
    OutOfMemory,
    #[error("operand can't have that rank")]
    RankError,
    #[error("J has attempted something insecure after you demanded heightened security")]
    SecurityViolation,
    #[error("You've . or : in the wrong place")]
    SpellingError,
    // #[error("During debugging: You tried to change the definition of a suspended entity")]
    #[error("Any time: Too many recursions took place")]
    StackError,
    #[error("Sentence has an unexecutable phrase")]
    SyntaxError,
    #[error("Execution took too long")]
    TimeLimit,
    #[error("There was no catcht. block to pick up your throw")]
    UncaughtThrow,
    #[error("that name has no value yet")]
    ValueError,

    #[error("shape error: {0}")]
    ShapeError(#[from] ndarray::ShapeError),

    #[error("{0} (legacy)")]
    Legacy(String),
}

impl JError {
    pub(crate) fn custom(message: impl ToString) -> anyhow::Error {
        anyhow::Error::from(JError::Legacy(message.to_string()))
    }
}
