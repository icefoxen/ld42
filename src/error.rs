//! Basic error types.

use ggez;

#[derive(Debug, Fail)]
pub enum Err {
    #[fail(display = "ggez error: {:?}", err)]
    GgezError { err: ggez::GameError },
}

impl From<ggez::GameError> for Err {
    fn from(err: ggez::GameError) -> Self {
        Err::GgezError { err }
    }
}
