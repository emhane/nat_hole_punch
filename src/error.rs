use rlp::DecoderError;
use std::{
    fmt,
    fmt::{Debug, Display},
};

#[macro_export]
macro_rules! impl_from_variant_wrap {
    ($(<$($generic: ident$(: $trait: ident$(+ $traits: ident)*)*,)+>)*, $from_type: ty, $to_type: ty, $variant: path) => {
        impl$(<$($generic $(: $trait $(+ $traits)*)*,)+>)* From<$from_type> for $to_type {
            fn from(e: $from_type) -> Self {
                $variant(e)
            }
        }
    };
}
#[macro_export]
macro_rules! impl_from_variant_unwrap {
    ($(<$($generic: ident$(: $trait: ident$(+ $traits: ident)*)*,)+>)*, $from_type: ty, $to_type: ty, $variant: path) => {
        impl$(<$($generic $(: $trait $(+ $traits)*)*,)+>)* From<$from_type> for $to_type {
            fn from(e: $from_type) -> Self {
                if let $variant(v) = e {
                    return v;
                }
                panic!("Bad impl of From")
            }
        }
    };
}

#[derive(Debug)]
pub enum HolePunchError<TDiscv5Error: Debug + Display> {
    NotificationError(DecoderError),
    InitiatorError(TDiscv5Error),
    RelayError(TDiscv5Error),
    TargetError(TDiscv5Error),
}

impl_from_variant_wrap!(<TDiscv5Error: Debug + Display,>, DecoderError, HolePunchError<TDiscv5Error>, Self::NotificationError);

impl<TDiscv5Error: Debug + Display> fmt::Display for HolePunchError<TDiscv5Error> {
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
