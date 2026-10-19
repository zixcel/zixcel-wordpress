# Security boundaries

Only an explicitly selected HTTPS origin, root WordPress installation and public
DNS addresses are accepted. Every resolved address is checked and pinned. IP
literals, nonstandard ports, URL credentials, path/query/fragment origins,
private/link-local/loopback/multicast and mapped-address destinations are refused.
Redirects and inherited proxies are disabled. DNS lookup uses the OS resolver;
the trusted command host should retain a process timeout in addition to HTTP budgets.

An unauthenticated discovery document must identify the selected origin and core
authorization endpoint before login. It is untrusted content and is not executed.
Authenticated responses are projected into explicit types. Provider errors expose
only controlled error enums/statuses, not URLs, response bodies or credentials.
The CLI also rejects output containing the supplied Application Password.

Basic headers are sensitive and temporary buffers use zeroize. This is not a
guarantee that every HTTP/TLS allocation or inherited process environment is
erased. The current ephemeral command model stores no login credential. Custody
integration and browser callbacks are unfinished; do not claim otherwise.

POST is submitted once, only after exact plan-hash approval and a durable journal.
An Application Password does not provide per-token scopes: use a suitable user.
The server remains authoritative for capability checks and input sanitization.
If WordPress changes draft text while sanitizing, exact read-back fails safely.
No rollback, delete, publish, plugin or theme update is attempted automatically.

The journal directory must be trusted and private. A hostile same-user process
is outside this command's isolation boundary. Keep live content and receipts out
of source control; review the selected target and draft content before approval.

Sources checked on 2026-10-19:
- https://developer.wordpress.org/rest-api/using-the-rest-api/authentication/
- https://developer.wordpress.org/rest-api/reference/users/
- https://developer.wordpress.org/rest-api/reference/posts/
