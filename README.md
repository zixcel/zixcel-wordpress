# Zixcel WordPress

Rust library and native `zix-wordpress` client for origin-bound WordPress core
REST authentication, permission discovery, bounded content reads and explicitly
approved creation of a non-public draft. Linux/WSL is the initial runtime.

Use the declared Rust toolchain, then `cargo test --locked --all-features` and
`cargo clippy --locked --all-targets --all-features -- -D warnings`. Linux builds
with `os-keyring` need the native libsecret development package. The CI release
contains the OS-custody-enabled GNU/Linux executable, dependency notices,
extension manifest, source binding and checksums; native GLib/libsecret remain
OS-managed runtime dependencies. Version tags publish through owner CI/OIDC.

## Connection

`zix-wordpress discover <profile-id> https://example.com` performs anonymous core
discovery and returns the site's authorization endpoint. It does not authenticate.
Use WordPress core's Application Password screen to approve an application for
a suitably restricted user. Do not provide the user's main login password,
extract browser cookies or install an authentication plugin.

For OS custody, provision an encrypted persistent Secret Service collection
using the OS provider, then explicitly unlock and enroll it. Credentials are
read as `{username,password}` JSON from stdin for import, never as arguments.
Use a trusted local hidden-input host, not a shell literal/history or LLM prompt.

```text
zix-wordpress credential-unlock <profile-id> https://example.com <collection>
zix-wordpress credential-import <profile-id> https://example.com <collection>
zix-wordpress credential-status <profile-id> https://example.com <collection>
zix-wordpress credential-lock <profile-id> https://example.com <collection>
```

Set the non-secret `ZIX_WORDPRESS_COLLECTION` alias for subsequent commands.
The credential is bound to both profile and exact origin; it is not obtained
from a profile file. Locked/missing/unavailable custody fails without falling
back to password environment variables. Unlocking affects the selected whole
collection and does not approve a write. `credential-forget` removes only the
local entry, not the remote Application Password. No secret-export CLI exists.
Software keyrings do not isolate malicious processes running as the same user.

An explicit ephemeral host may instead supply `ZIX_WORDPRESS_USERNAME` and
`ZIX_WORDPRESS_APPLICATION_PASSWORD` when no collection is selected. Keep them
out of argv, Git, profiles, model prompts and logs.

```text
zix-wordpress login <profile-id> https://example.com
zix-wordpress posts-list <profile-id> https://example.com
zix-wordpress pages-list <profile-id> https://example.com
zix-wordpress posts-get <profile-id> https://example.com <id>
zix-wordpress pages-get <profile-id> https://example.com <id>
```

Lists are explicit pages of at most 25 summaries (id, status and raw title), never
bulk article bodies. `get` retrieves one selected raw article. Nested `_fields`
avoid rendering and transferring unneeded HTML. All responses retain the same
1 MiB transport limit; an oversized article is reported rather than silently
raising the limit or retrying. A first-page read is not a complete site export.

Login verifies core discovery and authenticated `/users/me?context=edit`.
Application Password authentication grants that user's WordPress permissions;
the client does not claim server-enforced read-only token scopes.

## Explicit draft acceptance

`draft-plan <profile-id> <origin>` accepts `{kind,title,content}` JSON on stdin and
returns a ten-minute plan and its SHA256. Review the exact plan, then pass its
`plan` value on stdin to `draft-apply <profile-id> <origin> <approved-sha256>
<private-journal-directory>`. The journal must already exist, be owned by the
current user and have mode 0700. A plan is bound to origin, profile, user, content,
kind and a nonce. It creates one **draft**, then reads it back. It never publishes.

A durable exclusive journal is synced before POST. An uncertain result, partial
read-back or existing journal blocks replay across restarts. Reconcile with the
server before choosing further actions; a new plan is not an automatic retry.

## Editing and publication

`content-plan PROFILE ORIGIN` takes `{kind,effect,after}` JSON: `after` contains
`id,status,title,content`; effects are `edit_draft`, `publish`, `edit_published`.
The plan records the authenticated user and observed item/modified timestamp.
Review the whole before/after and effect; `content-apply PROFILE ORIGIN HASH
PRIVATE_JOURNAL_DIRECTORY` takes the exact plan on stdin. It checks current
permissions and pre-existing content/timestamp before writing and then reads back.
Drift is a conflict, not permission to overwrite. Core REST is not atomic CAS:
a writer can still race between the final read and POST.

`content-reconcile PROFILE ORIGIN` takes a prior plan and only observes whether
the desired state, unchanged snapshot or conflicting state is present. Matching
state does not prove which actor wrote it and never removes the replay barrier.
Lost write responses and mismatching read-back remain unknown. Reconciliation
never retries, publishes or rolls back. Installed zix mutation routing remains a
separate implementation; these are explicit native commands.

## Boundaries and remaining work

The Crowsi bounded sensitive-body collector is reused. The transport owns Basic
authentication, public DNS pinning, no inherited proxy, no redirects or retries,
HTTPS, response/time/request budgets and an allowlist of WordPress core paths.
Crowsi's existing bearer-only GET transport cannot perform these authenticated
draft operations, so it is not presented as a write adapter.

This is not browser `/wp-admin` session automation. It does not reuse browser
cookies. Browser dashboard login and the API's management operations require
separate live evidence. Browser callback authorization, remote revoke,
existing-draft updates, publish and media remain #160. Register the downloaded
`wordpress.extension.json` and executable with their verified hashes using
`zix extensions add`. The read-only installed extension requires a dispatcher
supporting explicit session-bus opt-in. It exposes `zix wordpress status`,
`zix wordpress posts list` and `zix wordpress pages list` with required
`--profile`, `--origin`, `--collection` arguments. It never auto-unlocks.
Writes are not registered as read-only commands in the common dispatcher.

No hosting, plugin, theme, IAM, DNS or production content is configured by tests.
Synthetic tests are not evidence of a customer's successful login or mutation.
See [security boundaries](SECURITY.md) for implementation and verification limits.
