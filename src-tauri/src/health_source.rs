//! The health inlet: one trait every health producer implements, and the
//! aggregator that merges them into a single [`HealthView`] (E021, ADR 020).
//!
//! Google Fit is a *source*, not the architecture. Fit shuts down in late
//! 2026, so the app talks to [`HealthSource`] and nothing else; replacing
//! Fit with an Android Health Connect companion (candidate E022) means
//! registering another source, not editing the UI or the IPC layer.
//!
//! Merge contract, implemented by [`HealthAggregator::merge`]:
//!   - `configured` is true when *any* source is configured;
//!   - the snapshot shown is the **freshest** one by `fetched_at_ms`;
//!   - an error from any source passes through, `AuthRevoked` first — a
//!     healthy source never hides another source's broken credentials.

use crate::health_models::{HealthErrorKind, HealthSnapshot, HealthView};
use async_trait::async_trait;
use std::sync::Arc;
use tokio::sync::RwLock;

/// A producer of today's health metrics.
///
/// `view` must not touch the network — it reports whatever the source has
/// cached, so UI reads stay instant. `refresh` is the call that may go out
/// to a network, and returns the post-refresh view.
#[async_trait]
pub trait HealthSource: Send + Sync {
    /// Stable identifier used as `HealthSnapshot::source_id`.
    ///
    /// Read by tests and by [`HealthAggregator::source_ids`] today; the
    /// push inlet (T03) labels its registered source with it.
    #[cfg_attr(not(test), allow(dead_code))]
    fn id(&self) -> &str;
    /// Cached state, no I/O.
    async fn view(&self) -> HealthView;
    /// Force a fresh read and return the resulting view.
    async fn refresh(&self) -> HealthView;
}

/// Fan-in over every registered [`HealthSource`].
#[derive(Default)]
pub struct HealthAggregator {
    sources: RwLock<Vec<Arc<dyn HealthSource>>>,
}

impl HealthAggregator {
    /// Build an aggregator over an initial set of sources.
    pub fn new(sources: Vec<Arc<dyn HealthSource>>) -> Self {
        Self {
            sources: RwLock::new(sources),
        }
    }

    /// Add a source after construction — used by the LAN push inlet, which
    /// only exists once the remote server is up (T03).
    #[cfg_attr(not(test), allow(dead_code))]
    pub async fn register(&self, source: Arc<dyn HealthSource>) {
        self.sources.write().await.push(source);
    }

    /// Identifiers of every registered source, in registration order.
    #[cfg_attr(not(test), allow(dead_code))]
    pub async fn source_ids(&self) -> Vec<String> {
        self.sources
            .read()
            .await
            .iter()
            .map(|s| s.id().to_string())
            .collect()
    }

    /// Merged cached view across all sources — no I/O.
    pub async fn view(&self) -> HealthView {
        let sources = self.sources.read().await.clone();
        let mut views = Vec::with_capacity(sources.len());
        for s in &sources {
            views.push(s.view().await);
        }
        Self::merge(views)
    }

    /// Refresh every source, then merge.
    pub async fn refresh(&self) -> HealthView {
        let sources = self.sources.read().await.clone();
        let mut views = Vec::with_capacity(sources.len());
        for s in &sources {
            views.push(s.refresh().await);
        }
        Self::merge(views)
    }

    /// Collapse per-source views into the one the UI renders.
    ///
    /// Freshness wins for the snapshot; errors are never swallowed.
    pub fn merge(views: Vec<HealthView>) -> HealthView {
        if views.is_empty() {
            return HealthView::unconfigured();
        }
        let configured = views.iter().any(|v| v.configured);
        let snapshot: Option<HealthSnapshot> = views
            .iter()
            .filter_map(|v| v.snapshot.clone())
            .max_by_key(|s| s.fetched_at_ms);
        // AuthRevoked outranks Transient: it is the only one the user can act on.
        let error = views
            .iter()
            .filter(|v| v.error_kind.is_some())
            .min_by_key(|v| match v.error_kind {
                Some(HealthErrorKind::AuthRevoked) => 0,
                _ => 1,
            });
        HealthView {
            configured,
            snapshot,
            error_kind: error.and_then(|v| v.error_kind),
            error_message: error.and_then(|v| v.error_message.clone()),
        }
    }
}

/// Type alias used by Tauri state.
pub type HealthState = Arc<HealthAggregator>;
