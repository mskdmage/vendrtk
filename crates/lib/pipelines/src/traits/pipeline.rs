use crate::error::Result;

pub trait Pipeline: Send + Sync {
    type Input;
    type Output;

    fn run(&self, input: Self::Input) -> impl Future<Output = Result<Self::Output>>;
}
