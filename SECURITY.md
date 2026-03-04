# Security Policy

## Supported Versions

The following versions of `wkhtmltopdf` (Rust rewrite) are currently supported
with security updates:

| Version | Supported          |
| ------- | ------------------ |
| main    | :white_check_mark: |

## Reporting a Vulnerability

If you discover a security vulnerability, **please do not open a public GitHub
issue**.  Instead, report it privately so that we can address it before public
disclosure.

**To report a vulnerability:**

1. Navigate to the repository's
   [Security Advisories](https://github.com/randymarsh77/wkhtmltopdf/security/advisories)
   page and select **"Report a vulnerability"**, **or**
2. Email the maintainer directly (see `AUTHORS` for contact information).

Please include as much of the following as possible:

- A description of the vulnerability and its potential impact.
- Steps to reproduce or a proof-of-concept.
- Affected versions / components.
- Any suggested mitigations or fixes.

We aim to acknowledge reports within **72 hours** and to release a fix or
advisory within **14 days** for confirmed vulnerabilities.

---

## Security Considerations

### Rendering untrusted HTML

`wkhtmltopdf` renders arbitrary HTML by launching a headless Chromium
subprocess.  When rendering HTML that you do not fully control, keep the
following in mind:

- **Sandbox isolation** – The headless Chromium subprocess is launched with the
  OS sandbox enabled by default (`HeadlessRenderer::sandbox = true`).  This
  confines the renderer process to a restricted set of kernel syscalls and
  filesystem views.  Do not disable the sandbox unless your environment
  explicitly prevents it (e.g. some container runtimes running as root), and
  only after understanding the security implications.

- **Mandatory Access Control** – For additional defence-in-depth, deploy an
  AppArmor or SELinux profile around the `wkhtmltopdf` binary.  See
  [`docs/apparmor.md`](docs/apparmor.md) for an AppArmor example profile.

- **JavaScript** – JavaScript is enabled by default because many pages require
  it to render correctly.  When rendering untrusted HTML, consider setting
  `enable_javascript = false` on `HeadlessRenderer` (or passing
  `--disable-javascript` / `--no-javascript` on the CLI).

### File-path handling

All local file paths passed to the converter (page sources, header/footer HTML
sources, and the outline dump target) are validated against **path traversal
attacks** before use.  Paths containing `..` (parent-directory) components are
rejected with an error.

When constructing file paths from user-supplied data, ensure the input is
properly sanitised before passing it to the library.

### Network inputs

Only `http://` and `https://` URL schemes are accepted for remote page
sources.  URLs with other schemes (e.g. `ftp://`, `javascript:`) are rejected
before the renderer is invoked.

TLS certificate verification is enabled by default.  It can be disabled via
`LoadPage::ssl_verify_peer` / `ssl_verify_host`, but this is strongly
discouraged in production environments.

### Proxy configuration

Proxy settings are passed through to the underlying HTTP client (`ureq`).
Ensure proxy URLs come from trusted configuration and not from user-supplied
request data.

### Dependency supply chain

This project uses `cargo audit` (via CI) to detect known vulnerabilities in
Rust dependencies.  Run `cargo audit` locally before publishing releases.
