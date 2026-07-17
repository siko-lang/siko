# Siko TLS experiment

This directory contains two standalone packages and is intentionally not wired into
`Std`, the compiler, or the repository test suite.

- `ssl/` is a direct Siko FFI binding and blocking TLS network layer over the existing
  `TcpStream` and `TcpListener` types.
- `demo/` depends on `ssl/` and performs an HTTPS/1.1 request through `TlsStream`.

The implementation links the already-installed OpenSSL 3 `ssl` and `crypto`
libraries. There is no C shim. The few header-only operations used by the experiment
(`SSL_CTX_set_*_proto_version`, SNI, and the memory-BIO EOF setting) are reimplemented
as their underlying control calls with constants copied into Siko.

Build and inspect the native link without making a network connection:

```text
./siko.bin build tls_experiment/demo -o /tmp/siko-tls-demo
/tmp/siko-tls-demo
```

Make an HTTPS request by supplying the literal IPv4 transport address separately
from the DNS name used for SNI and certificate verification:

```text
/tmp/siko-tls-demo 1.1.1.1 cloudflare.com /
```

The optional remaining arguments are a TCP port and `--insecure`. The latter exists
only to make local self-signed experiments possible and disables both chain and name
verification:

```text
/tmp/siko-tls-demo 127.0.0.1 localhost / 8443 --insecure
```

The current experiment deliberately has a narrow surface:

- safe defaults enable peer verification and require at least TLS 1.2;
- `Config` has Go-like server-name, verification, version, and certificate settings;
- client and server streams use two OpenSSL memory BIOs, while ciphertext I/O stays
  in Siko's existing TCP implementation;
- `TlsStream` implements `Reader`, `Writer`, and `Closer`, returning the standard
  dynamic `Error` object containing a concrete `Tls.Error`;
- `TlsListener.accept` creates a lazy-handshake server stream;
- server identity loading uses PEM file paths for this PoC; an eventual production
  API should own parsed certificate and private-key material;
- listener accepts compile an OpenSSL context per connection in this experiment;
  production code should compile one immutable server context and share it;
- `shutdown` sends and flushes `close_notify` but does not wait indefinitely for the
  peer's notification;
- there is no ALPN, client certificate authentication, session reuse, async wrapper,
  DNS resolution, or LibreSSL backend here.

Native resources use explicit, idempotent `close` operations because Siko has no
destructor convention for these wrappers yet. Callers must close streams and
listeners.
