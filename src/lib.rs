#![forbid(unsafe_code)]
//! WordPress owns site identity, core authentication, permissions and effect meaning.
mod apply;
mod client;
mod content;
mod content_apply;
#[cfg(all(target_os = "linux", feature = "os-keyring"))]
pub mod custody;
mod journal;
mod model;
mod mutation;
mod network;
mod plan;
mod request;
mod response;
mod routes;
mod snapshot;
mod transport;
pub use apply::apply_draft;
pub use client::WordPress;
pub use content_apply::{ContentReceipt, Reconciliation, apply_content, reconcile_content};
pub use model::{ContentItem, ContentKind, ContentSummary, Discovery, Permission, Profile};
pub use mutation::{ContentEffect, ContentPlan, Snapshot, plan_content};
pub use network::public_address;
pub use plan::{DraftPlan, DraftReceipt, plan_draft};
pub use transport::{Https, Method, Transport};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidProfile,
    InvalidCredential,
    InvalidResponse,
    InvalidInput,
    UnsafeTarget,
    Transport,
    Status(u16),
    ResponseLimit,
    Unsupported,
    PermissionDenied,
    PlanMismatch,
    Conflict,
    Replay,
    UnknownResult,
    LocalState,
}
pub type Result<T> = std::result::Result<T, Error>;
