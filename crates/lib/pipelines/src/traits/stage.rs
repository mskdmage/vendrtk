use crate::error::Result;
use crate::traits::context::Context;

pub trait Stage<C: Context>: Send + Sync {
    type Output;

    fn run(self, context: &mut C) -> impl Future<Output = Result<Self::Output>>;
}
