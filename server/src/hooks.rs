//! Server middleware hooks for plugin extensibility.
//!
//! Hooks allow server-side plugins to intercept document operations:
//! - Pre-save: validate/transform before writing to storage
//! - Post-save: trigger external actions after save
//! - Pre-export: transform document before format conversion
//!
//! Hooks are registered at startup and run synchronously in order.
//!
//! This module is not wired by default. Consumers register hooks at startup.
#![allow(dead_code)]

use std::sync::Arc;

/// Hook context passed to each hook function.
#[derive(Debug)]
pub struct HookContext {
    /// Document ID being operated on.
    pub doc_id: String,
    /// Document bytes (may be modified by pre-hooks).
    pub data: Vec<u8>,
    /// Format of the document.
    pub format: String,
    /// User performing the action (if authenticated).
    pub user_id: Option<String>,
    /// Metadata key-value pairs.
    pub metadata: std::collections::HashMap<String, String>,
}

/// Result of a hook execution.
pub enum HookResult {
    /// Continue processing (optionally with modified data).
    Continue(HookContext),
    /// Abort the operation with an error message.
    Abort(String),
}

/// A server-side hook function.
pub type HookFn = Arc<dyn Fn(HookContext) -> HookResult + Send + Sync>;

/// Registry of server hooks.
pub struct HookRegistry {
    pre_save: Vec<(String, HookFn)>,
    post_save: Vec<(String, HookFn)>,
    pre_export: Vec<(String, HookFn)>,
}

impl HookRegistry {
    pub fn new() -> Self {
        Self {
            pre_save: Vec::new(),
            post_save: Vec::new(),
            pre_export: Vec::new(),
        }
    }

    /// Register a pre-save hook.
    #[allow(dead_code)]
    pub fn on_pre_save(&mut self, name: &str, hook: HookFn) {
        self.pre_save.push((name.to_string(), hook));
        tracing::info!("Registered pre-save hook: {}", name);
    }

    /// Register a post-save hook.
    #[allow(dead_code)]
    pub fn on_post_save(&mut self, name: &str, hook: HookFn) {
        self.post_save.push((name.to_string(), hook));
        tracing::info!("Registered post-save hook: {}", name);
    }

    /// Register a pre-export hook.
    #[allow(dead_code)]
    pub fn on_pre_export(&mut self, name: &str, hook: HookFn) {
        self.pre_export.push((name.to_string(), hook));
        tracing::info!("Registered pre-export hook: {}", name);
    }

    /// Run all pre-save hooks. Returns modified context or error.
    #[allow(dead_code)]
    pub fn run_pre_save(&self, mut ctx: HookContext) -> Result<HookContext, String> {
        for (name, hook) in &self.pre_save {
            match hook(ctx) {
                HookResult::Continue(new_ctx) => ctx = new_ctx,
                HookResult::Abort(reason) => {
                    tracing::warn!("Pre-save hook '{}' aborted: {}", name, reason);
                    return Err(reason);
                }
            }
        }
        Ok(ctx)
    }

    /// Run all post-save hooks (fire-and-forget, errors logged).
    #[allow(dead_code)]
    pub fn run_post_save(&self, ctx: HookContext) {
        let mut current = ctx;
        for (name, hook) in &self.post_save {
            match hook(current) {
                HookResult::Continue(new_ctx) => current = new_ctx,
                HookResult::Abort(reason) => {
                    tracing::warn!("Post-save hook '{}' error: {}", name, reason);
                    return;
                }
            }
        }
    }

    /// Run all pre-export hooks. Returns modified context or error.
    #[allow(dead_code)]
    pub fn run_pre_export(&self, mut ctx: HookContext) -> Result<HookContext, String> {
        for (name, hook) in &self.pre_export {
            match hook(ctx) {
                HookResult::Continue(new_ctx) => ctx = new_ctx,
                HookResult::Abort(reason) => {
                    tracing::warn!("Pre-export hook '{}' aborted: {}", name, reason);
                    return Err(reason);
                }
            }
        }
        Ok(ctx)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pre_save_hook_modifies_data() {
        let mut registry = HookRegistry::new();
        registry.on_pre_save(
            "uppercase",
            Arc::new(|mut ctx| {
                ctx.data = ctx.data.iter().map(|b| b.to_ascii_uppercase()).collect();
                HookResult::Continue(ctx)
            }),
        );

        let ctx = HookContext {
            doc_id: "test".into(),
            data: b"hello".to_vec(),
            format: "txt".into(),
            user_id: None,
            metadata: Default::default(),
        };

        let result = registry.run_pre_save(ctx).unwrap();
        assert_eq!(result.data, b"HELLO");
    }

    #[test]
    fn pre_save_hook_can_abort() {
        let mut registry = HookRegistry::new();
        registry.on_pre_save(
            "reject-empty",
            Arc::new(|ctx| {
                if ctx.data.is_empty() {
                    HookResult::Abort("Document is empty".into())
                } else {
                    HookResult::Continue(ctx)
                }
            }),
        );

        let ctx = HookContext {
            doc_id: "test".into(),
            data: Vec::new(),
            format: "txt".into(),
            user_id: None,
            metadata: Default::default(),
        };

        let result = registry.run_pre_save(ctx);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Document is empty");
    }
}
