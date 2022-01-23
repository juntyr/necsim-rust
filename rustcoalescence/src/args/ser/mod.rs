use serde::{Serialize, Serializer};

mod r#impl;

use r#impl::{BufferingError, BufferingSerialize, BufferingSerializer};

#[derive(Clone)]
pub struct BufferingSerializeResult(Result<BufferingSerialize, BufferingError>);

impl<S: Serialize> From<&S> for BufferingSerializeResult {
    fn from(value: &S) -> Self {
        Self(value.serialize(BufferingSerializer))
    }
}

impl Serialize for BufferingSerializeResult {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match &self.0 {
            Ok(buffered) => buffered.serialize(serializer),
            Err(err) => Err(serde::ser::Error::custom(err)),
        }
    }
}
