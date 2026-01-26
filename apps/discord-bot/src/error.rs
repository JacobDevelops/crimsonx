#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("Discord API error: {0}")]
    Discord(#[from] Box<serenity::Error>),

    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),
}

impl From<serenity::Error> for Error {
    fn from(err: serenity::Error) -> Self {
        Error::Discord(Box::new(err))
    }
}

impl Error {
    pub fn user_message(&self) -> &str {
        match self {
            Error::Discord(_) => "Failed to communicate with Discord. Please try again.",
            Error::Config(msg) => msg,
            Error::Database(_) => "A database error occurred. Please try again later.",
        }
    }
}
