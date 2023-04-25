use rlp::DecoderError;
use std::fmt::{Debug, Display};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum HolePunchError<Discv5Error: Debug + Display> {
    #[error("error parsing notification")]
    NotificationError(#[from] DecoderError),
    #[error("failed initiating a hole punch attempt")]
    InitiatorError(Discv5Error),
    #[error("failed relaying a hole punch attempt")]
    RelayError(Discv5Error),
    #[error("failed as target of a hole punch attempt")]
    TargetError(Discv5Error),
}
