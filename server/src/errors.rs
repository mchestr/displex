use thiserror::Error;

// Make our own error that wraps `anyhow::Error`.
#[derive(Error, Debug)]
#[error(transparent)]
pub struct DisplexError(#[from] pub anyhow::Error);

impl From<async_graphql::Error> for DisplexError {
    fn from(err: async_graphql::Error) -> Self {
        Self(anyhow::Error::msg(err.message))
    }
}

impl From<serde_json::Error> for DisplexError {
    fn from(err: serde_json::Error) -> Self {
        Self(err.into())
    }
}

impl From<String> for DisplexError {
    fn from(err: String) -> Self {
        Self(anyhow::Error::msg(err))
    }
}

impl From<&'static str> for DisplexError {
    fn from(err: &'static str) -> Self {
        Self(anyhow::Error::msg(err))
    }
}
