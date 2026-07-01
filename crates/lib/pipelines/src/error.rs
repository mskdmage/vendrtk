pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("pipeline error")]
    Pipeline,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pipeline_error_display() {
        let err = Error::Pipeline;
        assert_eq!(err.to_string(), "pipeline error");
    }
}
