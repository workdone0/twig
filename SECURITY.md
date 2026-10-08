# Privacy and security

Twig processes files locally in a browser or terminal. The current Rust release
line receives security fixes; the historical Python branch is not actively
maintained here.

## Browser data

Files are processed in your browser and are not uploaded. The site downloads
its application assets, including JavaScript and WebAssembly, but document
contents are not included in requests. There are no analytics, accounts, or
third-party runtime scripts.

Each document stays in a worker's memory. Closing, replacing, cancelling, or
reloading the document ends that session. Twig does not persist documents or
selected paths. Only the Dark/Light preference is saved in local storage.
Browser extensions, clipboard managers, and operating-system behavior are
outside Twig's control; ending a session does not promise secure memory erasure.

Input, node, and allocation limits help bound resource use. They are not a hard
process-memory ceiling. See the [browser limits](docs/BROWSER.md#limits).

## Terminal data

The installed app makes no network or telemetry requests. Parsed document data
is stored in local, unencrypted SQLite caches; input snapshots and temporary
storage are also unencrypted. Unix cache permissions are restricted to the user;
Windows uses the user's profile permissions.

Use `--no-cache` to avoid publishing a persistent cache. This still uses temporary
disk storage, and abrupt termination can leave temporary files. Close other Twig
processes and use `twig --clear-cache` to remove stored caches. Neither command
promises secure erasure from disks or backups. Configuration is stored separately.
See [configuration and local data](README.md#configuration-and-local-data).

## Clipboard and output

Copying places data on the system clipboard, where other software may retain it.
CLI output can contain the complete input or selected data. Review where you
pipe, save, or share it, especially when working with HAR files or credentials.
The interactive explorers do not edit the original file; CLI output commands can
replace a destination you explicitly choose.

## Release verification

Release archives include SHA-256 checksums and GitHub provenance attestations.
Native OS code signing and notarization are not configured. Checksums verify
consistency with the published archive; trust still depends on its source and
distribution channel. The installers verify checksums and the executable version.

## Report a vulnerability

Use [GitHub private vulnerability reporting](https://github.com/workdone0/twig/security/advisories/new)
if available. If it is unavailable, open a minimal issue asking for a private
contact without disclosing exploitation details or sensitive data.

Include the Twig version, interface (web or terminal), operating system or browser,
steps to reproduce, and expected behavior. Use a small synthetic example. Never
attach production secrets, private HAR files, or customer records to a public issue.
