use std::sync::Arc;

use crate::traits::{Context, Stage};

pub struct Job<C: Context, S: Stage> {
    pub ctx: Arc<C>,
    pub stage: S,
}

impl<C: Context, S: Stage> Job<C, S> {
    pub fn new(ctx: Arc<C>, stage: S) -> Self {
        Self { ctx, stage }
    }
}
