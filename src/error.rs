use crate::impl_from_variant_wrap;
use rlp::DecoderError;
use std::{
    fmt,
    fmt::{Debug, Display},
};

#[derive(Debug)]
pub enum HolePunchError<TDiscv5Error: Debug + Display> {
    NotificationError(DecoderError),
    InitiatorError(TDiscv5Error),
    RelayError(TDiscv5Error),
    TargetError(TDiscv5Error),
}

impl_from_variant_wrap!(<TDiscv5Error: Debug + Display,>, DecoderError, HolePunchError<TDiscv5Error>, Self::NotificationError);

impl<TDiscv5Error: Debug + Display> Display for HolePunchError<TDiscv5Error> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HolePunchError::NotificationError(e) => {
                write!(f, "Error parsing notification. Error: {}", e)
            }
            HolePunchError::InitiatorError(e) => write!(
                f,
                "This node failed at initiating a hole punch attempt. Error: {}",
                e
            ),
            HolePunchError::RelayError(e) => write!(
                f,
                "This node failed at relaying a hole punch attempt. Error: {}",
                e
            ),
            HolePunchError::TargetError(e) => write!(
                f,
                "This node failed as the target of a hole punch attempt. Error: {}",
                e
            ),
        }
    }
}
