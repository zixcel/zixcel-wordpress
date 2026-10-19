use zixcel_wordpress::Error;
pub(crate) fn code(error: Error) -> &'static str {
    match error {
        Error::InvalidCredential | Error::Status(401) => "auth_required",
        Error::PermissionDenied | Error::Status(403) => "permission_denied",
        Error::Conflict => "conflict",
        Error::UnknownResult => "unknown_outcome",
        Error::Replay => "replay_refused",
        Error::PlanMismatch => "plan_mismatch",
        Error::ResponseLimit => "response_limit",
        Error::Status(429) => "rate_limited",
        Error::Unsupported => "unsupported",
        Error::Cancelled => "cancelled",
        Error::Expired => "expired",
        Error::Transport => "transport_failed",
        Error::LocalState => "custody_or_local_state_unavailable",
        Error::UnsafeTarget | Error::InvalidProfile => "unsafe_target",
        Error::Status(_) => "http_error",
        Error::InvalidResponse => "invalid_response",
        Error::InvalidInput => "invalid_input",
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn uncertain_and_rejected_requests_never_become_success_or_empty_lists() {
        assert_eq!(code(Error::UnknownResult), "unknown_outcome");
        assert_eq!(code(Error::Status(401)), "auth_required");
        assert_eq!(code(Error::Conflict), "conflict");
        assert_ne!(code(Error::Replay), code(Error::LocalState));
    }
}
