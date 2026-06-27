use std::future::Future;

use crate::error::Result;
use crate::traits::pipeline_context::PipelineContext;

pub trait PipelineStage<C: PipelineContext>: Send {
    type Output;

    fn run(self, ctx: &mut C) -> impl Future<Output = Result<Self::Output>> + Send;
}
