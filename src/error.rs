use rlp::DecoderError;
use std::fmt::{Debug, Display};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HolePunchError<Discv5Error: Debug + Display> {
    #[error("error parsing notification, {0}")]
    NotificationError(#[from] DecoderError),
    #[error("failed initiating a hole punch attempt, {0}")]
    InitiatorError(Discv5Error),
    #[error("failed relaying a hole punch attempt, {0}")]
    RelayError(Discv5Error),
    #[error("failed as target of a hole punch attempt, {0}")]
    TargetError(Discv5Error),
}
