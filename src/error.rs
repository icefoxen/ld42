//! Basic error types.

use ggez;
use specs;

#[derive(Debug, Fail)]
pub enum Err {
    #[fail(display = "ggez error: {:?}", err)]
    GgezError { err: ggez::GameError },

    #[fail(display = "specs error: {:?}", err)]
    SpecsError { err: specs::error::Error },
}

impl From<ggez::GameError> for Err {
    fn from(err: ggez::GameError) -> Self {
        Err::GgezError { err }
    }
}

impl From<specs::error::Error> for Err {
    fn from(err: specs::error::Error) -> Self {
        Err::SpecsError { err }
    }
}
