#![forbid(unsafe_code)]
//! WordPress owns site identity, core authentication, permissions and effect meaning.
mod apply;
mod authorization;
mod callback;
mod client;
mod content;
mod content_apply;
mod credential_identity;
#[cfg(all(target_os = "linux", feature = "os-keyring"))]
pub mod custody;
mod journal;
mod media;
mod model;
mod mutation;
mod network;
mod plan;
mod request;
mod response;
mod revocation;
mod routes;
mod snapshot;
mod transport;
mod upload_apply;
mod upload_plan;
pub use apply::apply_draft;
pub use authorization::{AuthorizationSession, BrowserGrant};
pub use client::WordPress;
pub use content_apply::{ContentReceipt, Reconciliation, apply_content, reconcile_content};
pub use credential_identity::CredentialIdentity;
pub use media::MediaItem;
pub use model::{ContentItem, ContentKind, ContentSummary, Discovery, Permission, Profile};
pub use mutation::{ContentEffect, ContentPlan, Snapshot, plan_content};
pub use network::public_address;
pub use plan::{DraftPlan, DraftReceipt, plan_draft};
pub use revocation::{RevokePlan, RevokeReceipt, apply_revoke, plan_revoke};
pub use transport::{Https, Method, Transport};
pub use upload_apply::{UploadReceipt, apply_upload};
pub use upload_plan::{UploadPlan, plan_upload};
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
    Expired,
    Cancelled,
    Replay,
    UnknownResult,
    LocalState,
}
pub type Result<T> = std::result::Result<T, Error>;
