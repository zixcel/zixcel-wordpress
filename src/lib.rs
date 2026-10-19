#![forbid(unsafe_code)]
//! WordPress owns site identity, core authentication, permissions and effect meaning.
mod apply;
mod client;
mod content;
mod model;
mod network;
mod plan;
mod request;
mod response;
mod routes;
mod transport;
pub use apply::apply_draft;
pub use client::WordPress;
pub use model::{ContentItem, ContentKind, ContentSummary, Discovery, Permission, Profile};
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
    Replay,
    UnknownResult,
    LocalState,
}
pub type Result<T> = std::result::Result<T, Error>;
