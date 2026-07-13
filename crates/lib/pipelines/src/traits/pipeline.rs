use std::sync::Arc;

use crate::traits::Context;

/// Owns shared context for a pipeline definition. Jobs borrow this via `Arc`.
pub struct Pipeline<C: Context> {
    ctx: Arc<C>,
}

impl<C: Context> Pipeline<C> {
    pub fn new(ctx: C) -> Self {
        Self { ctx: Arc::new(ctx) }
    }

    pub fn ctx(&self) -> Arc<C> {
        Arc::clone(&self.ctx)
    }
}
