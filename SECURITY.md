# Security

Twig processes untrusted local input. We support fixes on the current Rust
release line; the historical Python branch is not actively developed here.

Use GitHub's private vulnerability reporting if enabled:
https://github.com/workdone0/twig/security/advisories/new

If private reporting is unavailable, open a minimal issue asking for a private
contact without disclosing exploitation details or sensitive data. Never upload
production secrets, private HAR files, or real customer records in reproductions.

The explorer makes no network requests. SQLite caches and temporary input
snapshots are unencrypted. Use `--no-cache` to avoid persistent application caches;
close Twig and use `--clear-cache` to remove older caches. This does not promise
secure erasure from disks or backups. Clipboard data follows the host platform's
ownership and retention behavior.

Release archives have SHA-256 checksums and GitHub provenance attestations.
Native OS code signing and notarization are not currently configured. Checksums
verify consistency with published assets; trust still depends on the release
source and distribution channel.
