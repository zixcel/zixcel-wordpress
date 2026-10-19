# Zixcel WordPress

Rust library and native `zix-wordpress` client for origin-bound WordPress core
REST authentication, permission discovery, bounded content reads and explicitly
approved creation of a non-public draft. Linux/WSL is the initial runtime.

Use the declared Rust toolchain, then `cargo test --locked` and
`cargo clippy --locked --all-targets -- -D warnings`. The binary is built with
`cargo build --locked --bin zix-wordpress`.

## Connection

`zix-wordpress discover <profile-id> https://example.com` performs anonymous core
discovery and returns the site's authorization endpoint. It does not authenticate.
Use WordPress core's Application Password screen to approve an application for
a suitably restricted user. Do not provide the user's main login password,
extract browser cookies or install an authentication plugin.

The trusted local host supplies `ZIX_WORDPRESS_USERNAME` and
`ZIX_WORDPRESS_APPLICATION_PASSWORD` only to the command's environment. Do not
put credentials in argv, Git, profiles, model prompts or logs.

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

## Boundaries and remaining work

The Crowsi bounded sensitive-body collector is reused. The transport owns Basic
authentication, public DNS pinning, no inherited proxy, no redirects or retries,
HTTPS, response/time/request budgets and an allowlist of WordPress core paths.
Crowsi's existing bearer-only GET transport cannot perform these authenticated
draft operations, so it is not presented as a write adapter.

This is not browser `/wp-admin` session automation. It does not reuse browser
cookies. Browser dashboard login and the API's management operations require
separate live evidence. There is no persistent credential custody in this initial
slice. Browser callback authorization, remote revoke, existing-draft updates,
publish, media and installed `zix wordpress` protocol integration remain #160.
Writes are not registered as read-only commands in the common dispatcher.

No hosting, plugin, theme, IAM, DNS or production content is configured by tests.
Synthetic tests are not evidence of a customer's successful login or mutation.
See [security boundaries](SECURITY.md) for implementation and verification limits.
