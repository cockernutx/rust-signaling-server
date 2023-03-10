#[derive(Debug)]
pub enum Error {
    ParseError(serde_json::Error),
    ConnectionClosed,
    ConnectionTimeout,
    TargetNotFound(String),
    ServiceUnavailable,
    ServiceTimeout,
    UserAlreadyJoined(String)
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Error {
        Self::ParseError(err)
    }
}

impl std::fmt::Display for Error {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParseError(err) => write!(formatter, "ParseError({})", err),
            Self::ConnectionClosed => write!(formatter, "ConnectionClosed"),
            Self::ConnectionTimeout => write!(formatter, "ConnectionTimeout"),
            Self::TargetNotFound(target_user_name) => write!(
                formatter,
                "TargetNotFound(target_user_name: {})",
                target_user_name
            ),
            Self::ServiceUnavailable => write!(formatter, "ServiceUnavailable"),
            Self::ServiceTimeout => write!(formatter, "ServiceTemporaryUnavailable"),
            Self::UserAlreadyJoined(user) => write!(formatter, "UserAlreadyJoined({})", user),
        }
    }
}

impl std::error::Error for Error {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::ParseError(err) => Some(err),
            _ => None,
        }
    }
}