use std::future::Future;

use crate::error::Result;

pub trait Pipeline: Send {
    type Input;
    type Output;

    fn run(&mut self, input: Self::Input) -> impl Future<Output = Result<Self::Output>> + Send;
}
