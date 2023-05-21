// Make our own error that wraps `anyhow::Error`.
pub struct DisplexError(pub anyhow::Error);

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, DisplexError>`. That way you don't need to do that manually.
impl<E> From<E> for DisplexError
where
    E: Into<anyhow::Error>,
{
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
