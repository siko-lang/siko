# TLS network layer study

Date: 2026-07-16  
Status: TLS design study only; no TLS implementation or build files are changed

## 1. Executive conclusion

Siko can implement a TLS network layer directly over the existing `TcpStream` and
`TcpListener`, using OpenSSL through Siko FFI only. A C shim is not necessary. The
best initial architecture is:

- a separate `TlsStream` and `TlsListener` API under `Std.Net`, rather than changing
  the existing TCP types;
- a Go-inspired, backend-neutral `Tls.Config` which is validated and compiled into
  an immutable native context;
- a new `Tls.Error` enum with `Io(IO.Error)` and `Ssl(Tls.SslError)` variants which
  implements the standard `Error` trait;
- direct implementation of the standard `Reader`, `Writer`, and `Closer` traits,
  whose operations return the `Error` trait object;
- a blocking first implementation, with lazy server handshakes and explicit
  `handshake`, `read`, `write`, `shutdown`, and `close` operations;
- the already-installed Homebrew OpenSSL 3.6.2 as the PoC backend, without installing
  or replacing any native dependency;
- two OpenSSL memory BIOs per connection, so all ciphertext still moves through
  Siko's existing `TcpStream.read` and `TcpStream.write` methods;
- direct bindings for exported C functions and small Siko implementations of the
  header-only OpenSSL macros that are actually required;
- an independent native package for the OpenSSL backend, so importing ordinary
  `Std` does not force every Siko program to link `libssl` and `libcrypto`.

Using `SSL_set_fd` would be shorter, but it is the wrong primary design: it makes
OpenSSL perform socket I/O behind Siko's TCP abstraction, blurs which failures are
`IO.Error`, and makes later integration with Siko's async scheduler substantially
harder. A custom OpenSSL `BIO_METHOD` would preserve the abstraction but adds a large
and callback-heavy FFI surface. A pair of standard memory BIOs preserves the existing
TCP layer without either compromise.

LibreSSL can support the same public API in two ways:

1. Its low-level `libssl` API is close enough to use the same memory-BIO design. It
   should be a separate compile-time backend because LibreSSL and OpenSSL export many
   of the same symbol names but do not share an ABI.
2. LibreSSL's high-level `libtls` is a credible smaller implementation. Its
   `tls_connect_cbs` and `tls_accept_cbs` entry points can call Siko's `TcpStream`
   directly through `@c_callback`, still without a C shim. It has a much smaller FFI
   surface and secure defaults, but exposes less detailed TLS failure information and
   fewer advanced hooks. It is therefore a good reduced-scope backend, but low-level
   LibreSSL `libssl` is the closer match when identical semantics and structured SSL
   diagnostics matter.

Siko's standard I/O and effect boundary is intentionally open: fallible operations
return the concrete `Error` trait-object type. TLS preserves its domain model by
packing a complete `Tls.Error` into that object. A caller which does not care about
TLS distinctions uses the ordinary message/source interface. A caller which does
care narrows with `error is Tls.Error` and then matches the recovered enum
exhaustively. This keeps the effect API stable without pretending that one finite
enum can enumerate the failures of every user-provided effect implementation.

## 2. Scope and method

This study treats Siko source code and tests as authoritative. Existing prose
documentation is not used to infer current language or standard-library behavior.
The investigation covers:

- the current TCP, async TCP, I/O, FFI, callback, pointer, GC, and package-linking
  implementations;
- a concrete public API resembling the useful parts of Go's `crypto/tls` API;
- connection ownership, handshaking, shutdown, certificate and trust handling,
  hostname verification, ALPN, errors, and native-resource cleanup;
- the direct installed OpenSSL 3 FFI surface and header macros which must be translated into
  Siko;
- equivalent low-level LibreSSL and high-level `libtls` implementations;
- package layout, implementation phases, and tests.

It intentionally does not design DTLS, QUIC TLS, STARTTLS protocol policy, a generic
pluggable transport framework, or an async public API for the first release.

## 3. What the current Siko source establishes

### 3.1 TCP is already the right transport foundation

[`std/Common/Net/TcpStream.sk`](std/Common/Net/TcpStream.sk) owns a
`FileDescriptor`, exposes it through `get_fd`, and provides blocking `connect`,
`read`, `write`, `close`, and `set_nonblocking` operations. It implements the current
`Reader`, `Writer`, and `Closer` traits. [`std/Common/Net/TcpListener.sk`](std/Common/Net/TcpListener.sk)
likewise owns a descriptor and returns an ordinary `TcpStream` from `accept`.

That is enough for TLS. The TLS layer should own a `TcpStream`, not copy out its file
descriptor and create a second socket owner. `TlsListener` should similarly own a
`TcpListener`, call its existing `accept`, and wrap the returned stream.

There are two transport limitations which TLS should preserve rather than disguise:

- the current `connect` address parser accepts literal IPv4 addresses; it is not a
  DNS resolver;
- the current listener binds an IPv4 wildcard address and takes only a port.

Consequently, `TlsStream.connect(host, port, config)` may derive a verification name
from `host`, but it must not claim to add name resolution. A caller connecting to an
IP address for a certificate issued to a DNS name must supply `config.server_name`
explicitly and arrange address resolution separately.

### 3.2 Async TCP proves the direction, but is not the first TLS API

[`std/Common/Net/Async/TcpStream.sk`](std/Common/Net/Async/TcpStream.sk) wraps the
base TCP stream and waits for scheduler readiness when an operation returns
`EAGAIN`. [`std/Common/Net/Async/TcpListener.sk`](std/Common/Net/Async/TcpListener.sk)
does the same for accept and sets accepted streams nonblocking.

TLS operations do not have a fixed direction: a TLS read can require a socket write,
and a TLS write can require a socket read. OpenSSL reports those states as
`SSL_ERROR_WANT_READ` and `SSL_ERROR_WANT_WRITE`. The proposed memory-BIO engine
preserves those states internally, which makes a later async implementation feasible,
but a blocking first version avoids prematurely coupling TLS to the current event-loop
API.

Passing a nonblocking `TcpStream` to the blocking TLS constructor should be documented
as unsupported in version one. If it returns `EAGAIN`, the error can be preserved as
`Tls.Error.Io`, but the blocking engine must not pretend that the handshake completed.

### 3.3 The standard `Error` object is the I/O and effect boundary

[`std/Common/IO/Reader.sk`](std/Common/IO/Reader.sk),
[`std/Common/IO/Writer.sk`](std/Common/IO/Writer.sk), and
[`std/Common/IO/Closer.sk`](std/Common/IO/Closer.sk) return the `Std.Error` trait
object. Concrete filesystem and network operations use the same boundary and pack
`IO.Error` values into it. TLS can therefore add protocol alerts, certificate
validation failures, malformed peer messages, unexpected EOF, and native-library
failures without inventing errno values.

Making the I/O traits generic over their error would propagate an error parameter
through every generic reader and writer. More importantly, it conflicts with the
purpose of Siko's effect interfaces: an effect API is meant to remain concrete and
stable while the user supplies its implementation. `Error` is itself a concrete,
stable return type, while the value stored inside it remains open to effect handlers.

`TlsStream` should consequently implement the ordinary traits directly:

```siko
instance Reader[TlsStream] {
    fn read(stream: TlsStream, buffer: Slice[U8]) -> Result[Int, Error] {
        // perform TLS read; concrete failures use Err(tls_error).context()
    }
}

instance Writer[TlsStream] { /* same Error boundary */ }
instance Closer[TlsStream] { /* same Error boundary */ }
```

The TLS engine converts a TCP failure to `Tls.Error.Io` after narrowing the dynamic
source with `error is IO.Error`; native and protocol failures become
`Tls.Error.Ssl`. It then packs the complete `Tls.Error` into the return object. A
caller can recover exhaustive domain handling when needed:

```siko
match stream.read(buffer) {
    Ok(count) => use(count),
    Err(error) => {
        if error is Tls.Error {
            match error {
                Tls.Error.Io(error) => handle_io(error),
                Tls.Error.Ssl(error) => handle_tls(error)
            }
        } else {
            handle_other(error)
        }
    }
}
```

The exhaustive guarantee applies after the dynamic value has been narrowed to
`Tls.Error`; it does not make a false claim about the infinite set of error types an
effect implementation may return.

### 3.4 Concrete errors are retained, not flattened

[`std/Common/Http/Client.sk`](std/Common/Http/Client.sk),
[`std/Common/Archive/Tar.sk`](std/Common/Archive/Tar.sk), and
[`std/Common/Compression/Gzip.sk`](std/Common/Compression/Gzip.sk) define higher-level
error enums which retain a dynamic I/O source. `Tls.Error` is more specific because
its transport is the concrete `TcpStream`: normal transport failures can be narrowed
to `IO.Error` and stored losslessly in `Tls.Error.Io`.

[`std/Common/IO/Error.sk`](std/Common/IO/Error.sk) stores an errno and a message. It
cannot naturally express a zero-length successful-looking write, a truncated TLS
record, or a verification failure. Those conditions remain concrete `Tls.Error`
variants rather than being flattened into `IO.Error` or a string.

Both `IO.Error` and `Tls.Error` should implement the existing
[`Std.Error`](std/Common/Error.sk) trait. That source already provides trait-object
packing and error chains; the
[`trait_objects`](test/success/std/trait_objects/main.sk) and
[`trait_object_is`](test/success/std/trait_object_is/main.sk) tests exercise the
relevant runtime representation and narrowing. The
[`error_trait_object`](test/success/std/effects/error_trait_object/main.sk) success
test additionally proves that one effect returning `Error` can use handlers which
produce different concrete error types. `Result.context()` without a description
performs the standard packing; with a description it also adds a context source.

Rust makes the open-ended side of this trade for all standard streams: its `Read`
and `Write` traits return the concrete `std::io::Error`; rustls streams and gzip
writers conform to those traits, while `io::Error` can retain a custom error payload
for later inspection or downcasting. Siko places the open trait object one level
higher, so domain libraries do not need to force unrelated failures into an I/O
error wrapper.

### 3.5 Direct callbacks and raw FFI are available

The source contains direct `@extern("C")` declarations, renamed symbols, raw pointers,
`void*`, function pointer types (`fn*(...)`), allocation, and `transmute`; the pointer
operations are visible in [`std/Common/Ptr.sk`](std/Common/Ptr.sk). Most
importantly, [`std/Common/Sync/Thread.sk`](std/Common/Sync/Thread.sk) defines a Siko
function with `@c_callback` and passes it directly to `pthread_create`. Compiler
parsing and specialization for that annotation are present in
[`siko/Common/Parser/Annotations.sk`](siko/Common/Parser/Annotations.sk) and
[`siko/Compiler/Monomorphizer/Specialize.sk`](siko/Compiler/Monomorphizer/Specialize.sk).

Therefore OpenSSL verification/ALPN/SNI callbacks, or LibreSSL `libtls` transport
callbacks, can be implemented directly in Siko. No C trampoline is needed.

Callback rules should be explicit:

- no Siko panic or unwinding may cross the C boundary;
- callback state must remain alive for as long as the native library stores its
  pointer;
- borrowed native pointers must be copied before the operation that invalidates them;
- the exact C integer, `size_t`, pointer, and function-pointer ABI must be represented;
- callbacks should catch their own logical errors and return the native failure code.

The currently supported platform code is Darwin/Linux and 64-bit oriented, so the
initial binding may target their LP64 ABI. Windows/LLP64 should not be advertised
without an explicit type audit.

### 3.6 GC and resource lifetime require deliberate wrappers

[`std/Common/GC/GC.sk`](std/Common/GC/GC.sk) and compiler linkage show that Siko uses
the Boehm collector. Its non-moving allocation is helpful for native callbacks, but a
Siko object referenced only by a pointer stored in native OpenSSL memory should not be
assumed to remain a collector root. `TlsContext` or `TlsStream` must also retain the
Siko callback state.

[`std/Common/Vec.sk`](std/Common/Vec.sk) reallocates storage as it grows. A native
pointer into a vector must never survive an operation which can grow that vector.
ALPN callback output can safely point into the peer's offered protocol buffer for the
duration allowed by OpenSSL; configuration-owned byte buffers passed for longer
storage need fixed lifetime and an explicit owner.

[`std/Common/Libc/String.sk`](std/Common/Libc/String.sk) can create NUL-terminated C
strings, but does not by itself make an embedded NUL a valid configuration value.
Server names, cipher expressions, file names, and other C strings must be checked for
embedded NUL before conversion.

Current standard-library resource types generally rely on explicit `close`/`free`
and `defer`, not a language destructor. TLS safe wrappers should consequently carry a
closed/freed state and make cleanup idempotent. The ownership chain should be:

```text
TlsListener
  owns TcpListener
  owns shared native TlsContext

TlsStream
  owns TcpStream
  owns native SSL/tls connection
  owns callback state and fixed buffers
  indirectly owns BIOs after attaching them to SSL
```

No public method should expose a second owning alias to the underlying TCP stream or
native pointers.

### 3.7 Native libraries belong in their own package

[`siko/Common/Package/Parser.sk`](siko/Common/Package/Parser.sk) parses package-native
library candidates. [`siko/Compiler/Passes.sk`](siko/Compiler/Passes.sk) resolves a
candidate directory and sends `-L`, `-l`, and an rpath to the Clang linker. The LLVM
dependency in [`siko/Compiler/package.toml`](siko/Compiler/package.toml) is a concrete
example.

Putting `ssl` and `crypto` directly into `std/Common/package.toml` would make ordinary
`Std` consumers inherit those native dependencies. The backend should instead be a
separate Siko package which depends on `Std` and contributes modules in the `Std.Net`
namespace, in the same broad style that other repository packages extend `Std`.

OpenSSL and LibreSSL must be different packages/build selections. Both commonly ship
libraries named `ssl` and `crypto`, and their similar symbols are not evidence of ABI
compatibility. They must never be intentionally linked into the same Siko process.

## 4. Proposed public API

The goal is to resemble Go in configuration vocabulary and lifecycle, not to copy
every field or its exact mutability semantics. Siko should expose a smaller safe core
whose fields have the same meaning on both backends.

### 4.1 Shared TLS types

The following is illustrative Siko API syntax, not a claim that the declarations can
be pasted into the compiler unchanged:

```siko
module Std.Net.Tls

pub enum Version {
    Tls12,
    Tls13,
}

pub enum ClientAuthType {
    NoClientCert,
    RequestClientCert,
    RequireAnyClientCert,
    VerifyClientCertIfGiven,
    RequireAndVerifyClientCert,
}

pub struct Certificate {
    // Backend-neutral, owned certificate-chain and private-key material.
    // No public native handles.
}

pub struct CertPool {
    // Backend-neutral, owned trust anchors.
}

pub struct CipherSuite {
    pub id: U16, // IANA cipher-suite identifier
}

pub struct Config {
    pub certificates: Vec[Certificate],
    pub root_cas: Option[CertPool],
    pub next_protocols: Vec[String],
    pub server_name: Option[String],
    pub client_auth: ClientAuthType,
    pub client_cas: Option[CertPool],
    pub insecure_skip_verify: Bool,
    pub min_version: Version,
    pub max_version: Option[Version],
    pub cipher_suites: Vec[CipherSuite],
}
```

`Config.new()` should produce safe defaults:

- peer verification enabled;
- minimum TLS 1.2;
- maximum TLS 1.3/current supported maximum;
- backend default cipher policy when `cipher_suites` is empty;
- no ALPN protocols;
- no certificate;
- no client-certificate request on servers.

The initial `Version` enum should expose only TLS 1.2 and TLS 1.3. SSLv2, SSLv3,
TLS 1.0, and TLS 1.1 should not be representable through the safe API. Backend raw
constants remain internal.

`cipher_suites` should follow modern Go semantics: it controls TLS 1.2 suites, while
TLS 1.3 retains the library's secure default suites. Entries are IANA identifiers,
not backend cipher-name strings. Each backend maps the supported IDs to its own
configuration syntax and rejects an unsupported ID during context compilation. An
empty list means “do not call a cipher override API.” As in Go, list order should not
promise cipher preference; treat it as an enabled set and let the backend's secure
selection policy choose among mutual suites.

Fields worth considering after the core is stable include curve preferences,
disabled session tickets, key logging behind an explicitly dangerous API, multiple
certificate selection, and session caches. Go's callback-heavy certificate selection,
custom verification hooks, ECH, QUIC fields, and internal session machinery should
not be copied into version one merely for surface similarity.

The intended relationship to Go's `tls.Config` is explicit:

| Go concept | Initial Siko form | Decision |
|---|---|---|
| `Certificates` | `certificates` | included for server and optional client identity |
| `RootCAs` | `root_cas` | included; `None` selects the backend's documented default trust source |
| `NextProtos` | `next_protocols` | included as ALPN |
| `ServerName` | `server_name` | included for SNI and identity verification, with IP-specific handling |
| `ClientAuth` | `client_auth` | included with Go-like enum names |
| `ClientCAs` | `client_cas` | included for verified client authentication |
| `InsecureSkipVerify` | `insecure_skip_verify` | included, deliberately conspicuous |
| `MinVersion`, `MaxVersion` | version enum fields | included, but unsafe legacy versions are not representable |
| `CipherSuites` | IANA-ID list | included for TLS 1.2 only, matching modern Go meaning |
| `GetCertificate`, `GetClientCertificate` | none initially | defer multiple/dynamic identity selection |
| `VerifyPeerCertificate`, `VerifyConnection` | none initially | defer until callback safety and portable semantics are designed |
| `ClientSessionCache`, session wrap/unwrap | none initially | defer reusable client contexts and session policy |
| ticket controls | none initially | use safe native defaults, then add explicit policy |
| `KeyLogWriter` | none initially | omit secret-export surface from the safe core |
| curve/signature preferences | none initially | use backend security defaults |
| ECH, QUIC, renegotiation knobs | none | out of scope for this TCP TLS layer |

`root_cas = None` should concretely mean “load the selected backend's documented
default verification paths.” Failure to load them is a configuration error, not an
empty successful store. `Some(CertPool)` replaces that source with the explicit pool.
An explicit empty pool therefore trusts no certificate. This is Go-like at the API
level, while the operational meaning of “system” is limited by the chosen native
distribution as discussed below.

### 4.2 Certificate and trust construction

Backend-neutral constructors keep native lifetimes out of user code:

```siko
Certificate.load_x509_key_pair(
    certificate_pem: Slice[U8],
    private_key_pem: Slice[U8]
) -> Result[Certificate, Error]

CertPool.new() -> CertPool
pool.append_pem(pem: Slice[U8]) -> Result[(), Error]
pool.add_der(der: Slice[U8]) -> Result[(), Error]
```

The certificate loader should validate that the private key matches the leaf
certificate and retain the complete chain. Internally it can normalize the material
to owned DER. Context compilation then parses the owned bytes into temporary native
`X509`/`EVP_PKEY` objects, installs them into `SSL_CTX`, and frees the temporaries
according to the documented reference-counting rules.

Like Go's in-memory key-pair helper, the first API should accept unencrypted private
key material and return a clear parse/configuration error for encrypted PEM rather
than introduce a password callback. A later password-taking loader can decrypt into
owned temporary memory and erase that temporary buffer as far as the runtime permits.
Supported RSA, ECDSA, and Ed25519 key forms must be defined as the tested intersection
of the selected backend builds; a successfully parsed but unsupported signing key is
a construction error, not a delayed handshake surprise.

This approach has two advantages over keeping native certificate objects in public
values: it is backend-neutral, and it avoids requiring destructors for every copied
configuration object. As in Go, callers should still treat `Certificate` as sensitive
because it retains private-key material in process memory.

`CertPool` should deduplicate certificates by their canonical DER (or a collision-safe
DER key) before native compilation. OpenSSL reports a duplicate added through
`X509_STORE_add_cert` as a queue error; preventing the duplicate in Siko avoids having
to treat and clear that particular native failure as expected. Each native store gets
its own references, so freeing a temporary parsed `X509` never leaves the store with a
dangling object.

Loading system roots should be an explicit operation or the default only where its
behavior is documented per platform. `SSL_CTX_set_default_verify_paths` uses the
OpenSSL installation's configured trust locations; it does not automatically mean
the macOS Keychain or every host operating system's native trust policy. Custom
`root_cas` must work independently of system-root integration.

### 4.3 Stream API

```siko
module Std.Net.TlsStream

pub struct TlsStream {
    // private
}

pub fn client(
    stream: TcpStream,
    config: Tls.Config
) -> Result[TlsStream, Error]

pub fn server(
    stream: TcpStream,
    config: Tls.Config
) -> Result[TlsStream, Error]

pub fn connect(
    host: String,
    port: U16,
    config: Tls.Config
) -> Result[TlsStream, Error]

pub fn handshake(stream: TlsStream) -> Result[(), Error]
pub fn read(stream: TlsStream, buffer: Slice[U8]) -> Result[Int, Error]
pub fn write(stream: TlsStream, buffer: Slice[U8]) -> Result[Int, Error]
pub fn shutdown(stream: TlsStream) -> Result[(), Error]
pub fn close(stream: TlsStream) -> Result[(), Error]
pub fn connection_state(stream: TlsStream) -> Tls.ConnectionState
```

`client` and `server` wrap an already-connected stream. `connect` is only a
convenience combining the existing TCP connect with `client`; it does not perform
DNS. When `server_name` is absent, `connect` derives it from `host`. The arbitrary
stream `client` constructor must reject a verified configuration without a server
name, because silently omitting hostname verification is unsafe.

For a DNS name, the backend sends SNI and verifies that name. For an IP literal it
performs IP subject-alternative-name verification and does not send DNS SNI. Setting
`insecure_skip_verify` disables chain and name verification and should remain
conspicuously named and documented as unsafe.

`read` and `write` call `handshake` lazily if needed, matching the convenient part of
Go's behavior. `handshake` remains public so callers can force authentication before
passing a stream to application code.

`TlsStream` implements the ordinary `Std.IO.Reader`, `Writer`, and `Closer` traits
directly. Inherent methods and trait methods share the same `Error` return boundary
and preserve concrete TLS failures by packing `Tls.Error`; no compatibility wrapper
or parallel TLS I/O traits are needed.

Zero-length reads and writes return `Ok(0)` without forcing a handshake; the first
non-empty I/O operation performs the lazy handshake. This avoids surprising network
traffic from a no-op buffer probe.

`ConnectionState` should contain backend-neutral copied values:

```siko
pub struct ConnectionState {
    pub handshake_complete: Bool,
    pub version: Option[Version],
    pub did_resume: Bool,
    pub cipher_suite: Option[CipherSuite],
    pub negotiated_protocol: Option[String],
    pub server_name: Option[String],
    pub peer_certificates: Vec[PeerCertificate],
    pub verified_chains: Vec[Vec[PeerCertificate]],
}
```

Any pointers returned by native inspection functions are borrowed. The implementation
must copy strings and certificate bytes before the next native operation or before
freeing the connection.

### 4.4 Listener API

```siko
module Std.Net.TlsListener

pub struct TlsListener {
    // owns a TcpListener and compiled server context
}

pub fn bind(
    port: U16,
    config: Tls.Config
) -> Result[TlsListener, Error]

pub fn from_tcp_listener(
    listener: TcpListener,
    config: Tls.Config
) -> Result[TlsListener, Error]

pub fn accept(listener: TlsListener) -> Result[TlsStream, Error]
pub fn accept_and_handshake(listener: TlsListener) -> Result[TlsStream, Error]
pub fn close(listener: TlsListener) -> Result[(), Error]
```

The server config is validated and compiled once when the listener is built.
`accept` performs only TCP accept plus cheap per-connection SSL allocation and returns
a stream whose TLS handshake is lazy. This prevents one slow or malicious handshake
from blocking the listener's accept loop. `accept_and_handshake` is a convenience for
simple sequential servers, not the primitive on which `accept` is built.

At least one usable server certificate is required in version one. Supporting
multiple certificates selected by SNI is a later addition, implemented with a direct
Siko callback while keeping the certificate-selection state rooted by the context.
To retain Go-shaped config without ambiguous behavior, version one should require
exactly one server certificate and allow at most one client certificate; it rejects
larger vectors during role-specific config compilation rather than silently choosing
the first entry.

### 4.5 Config compilation and mutability

Native `SSL_CTX` values should be immutable after they become shared by accepted
connections. The public `Config` is validated and compiled into a private
`TlsContext` snapshot at `TlsListener` construction or at client stream construction.
Subsequent changes to the caller's config value do not mutate live connections.

Compiling one client context per `TlsStream.client` call is correct but potentially
expensive. A later `TlsConnector`/`ClientContext` can compile and safely reuse a
client configuration across connections. It is better to add that optimization as a
separate public type than to expose mutable native contexts in `Config`.

## 5. Error model

### 5.1 Concrete TLS shape

The top-level error enum should have exactly the requested separation:

```siko
pub enum Error {
    Io(IO.Error),
    Ssl(SslError),
}

instance Std.Error.Error[Error] {
    fn message(error: Error) -> String {
        error.to_string()
    }
}

instance Into[IO.Error, Error] {
    fn into(error: IO.Error) -> Error {
        Error.Io(error)
    }
}

pub enum SslError {
    Library(SslLibraryError),
    Verification(VerificationError),
    InvalidConfiguration(String),
    UnexpectedEof,
    WriteZero,
    Closed,
    UnsupportedState(String),
    UnsupportedRetryState(String),
}
```

`SslError` is named for the native SSL/TLS layer, but it remains backend-neutral.
`Library` contains diagnostics captured from OpenSSL or LibreSSL. `UnexpectedEof`
means the TCP transport ended without an authenticated TLS `close_notify`; it is not
equivalent to clean EOF. `WriteZero` covers an underlying write which returns zero
without an I/O error. `Closed` makes use-after-close deterministic.

```siko
pub struct SslLibraryError {
    pub operation: Operation,
    pub class: SslErrorClass,
    pub stack: Vec[SslErrorEntry],
}

pub struct VerificationError {
    pub operation: Operation,
    pub code: I64,
    pub text: String,
    pub stack: Vec[SslErrorEntry],
}

pub enum Operation {
    Configure,
    ParseCertificate,
    Handshake,
    Read,
    Write,
    Shutdown,
}

pub enum SslErrorClass {
    Protocol,
    Syscall,
    Internal,
}

pub struct SslErrorEntry {
    pub code: U64,
    pub text: String,
    pub library: Option[String],
    pub reason: Option[String],
    pub function: Option[String],
}
```

File and line information from a native library build is generally not useful as a
stable public contract. It may be retained for debugging, but portable code should
use operation, class, code, and text. OpenSSL 3 can supply function information;
LibreSSL's older queue API may not.

A fatal native result can legitimately have an empty queue. In that case the adapter
creates one backend-neutral diagnostic entry from the `SSL_get_error` class,
operation, transport-EOF state, and verification result rather than returning an
unexplained empty `Library` value.

Every public TLS failure is returned as the standard `Error` object by calling
`Err(tls_error).context()`. This packing does not change the concrete value. The
caller can use `is Tls.Error` and then match `Io` and `Ssl` exhaustively.

### 5.2 OpenSSL error discipline

OpenSSL's error queue is per-thread and `SSL_get_error` depends on both the exact
operation return value and the queue. Every TLS operation must follow this sequence:

1. call `ERR_clear_error` immediately before the `SSL_*` operation;
2. call the operation and retain its exact return value;
3. on a non-success result, call `SSL_get_error(ssl, result)` immediately, on the same
   thread, with no intervening OpenSSL call;
4. classify `WANT_READ`, `WANT_WRITE`, `ZERO_RETURN`, `SYSCALL`, and `SSL`;
5. only then drain and copy the native error queue into Siko values.

Configuration functions which do not use `SSL_get_error` still clear the queue before
the call and drain it on failure. Stale queue entries must never be attached to the
next failure.

`SSL_ERROR_WANT_READ` and `SSL_ERROR_WANT_WRITE` are engine states, not public errors
for a blocking stream. Other retry classes such as `WANT_X509_LOOKUP`, async-engine
states, `WANT_RETRY_VERIFY`, or client-hello callback suspension are unsupported until
the engine knows how to resume them; they become `UnsupportedRetryState` rather than
a busy loop.

After a failed verified handshake, the backend also captures `SSL_get_verify_result`
and copies `X509_verify_cert_error_string`. The verification result is useful even
when the main OpenSSL error stack is terse.

Classification is deterministic: when verification was required and the native
verify result is not `X509_V_OK`, return `SslError.Verification` and attach the copied
native queue to that value. Otherwise a fatal `SSL_ERROR_SSL`/`SYSCALL` becomes
`SslError.Library`. The unverified client-certificate modes deliberately do not turn
their saved non-OK native verification result into a handshake error.

The installed OpenSSL 3.6.2 provides `ERR_get_error_all`; LibreSSL uses the older
`ERR_get_error_line_data` family. A backend adapter should normalize either source
into `SslErrorEntry` and copy all C strings immediately.

### 5.3 Preserving real I/O failures

The memory-BIO architecture means OpenSSL never performs the socket syscall. A
failure from `TcpStream.read` or `TcpStream.write` is therefore known directly and is
returned as `Error.Io` with the original errno and message. `SSL_ERROR_SYSCALL` is
reserved for a native inconsistency or EOF classification, not used as a lossy proxy
for Siko socket errors.

For the `libtls` callback alternative, each transport callback stores the exact
`IO.Error` in callback state before returning native failure. The outer TLS operation
clears that slot before starting and checks it before converting `tls_error` into an
SSL error. This out-of-band channel is required because `libtls` intentionally exposes
mainly an error string and does not promise to preserve useful `errno` state.

## 6. Connection engine over memory BIOs

### 6.1 Native object graph

For the OpenSSL and low-level LibreSSL backends, each connection has:

```text
                          plaintext
application  <-------------------------------->  SSL object
                                                     |
                            +------------------------+-----------------------+
                            |                                                |
                        read BIO                                         write BIO
                            ^                                                |
                            | ciphertext                                     | ciphertext
                            |                                                v
                         TcpStream.read                              TcpStream.write
```

The read BIO contains ciphertext received from TCP but not yet consumed by TLS. The
write BIO contains ciphertext produced by TLS but not yet sent to TCP. Both are
standard memory BIOs created with `BIO_s_mem`; there is no custom BIO implementation.

On the installed OpenSSL 3.6.2, `SSL_set0_rbio` and `SSL_set0_wbio` are preferable to `SSL_set_bio`
because each call has a simple ownership transfer. Once attached, the `SSL` object
owns and frees the BIOs. The LibreSSL backend should hide its equivalent attachment
operation behind the same internal interface. The LibreSSL 4.3 adapter uses
`SSL_set_bio`, whose more complicated reference-counting rules stay isolated inside
that adapter.

Construction must be failure-atomic. Before ownership transfers, the constructor
frees each successfully allocated object on later failure. After transfer, it frees
only the `SSL` object. The safe wrapper records the point of transfer so an error path
cannot double-free a BIO.

The available low-level transport choices compare as follows:

| Choice | Why it is not the initial design |
|---|---|
| `SSL_set_fd` / socket BIO | OpenSSL bypasses `TcpStream.read`/`write`, so Siko loses direct I/O errors and future scheduler control |
| custom `BIO_METHOD` | preserves TCP, but requires several C callbacks, retry-flag control, foreign state, and more lifetime/error machinery |
| `BIO_new_bio_pair` | built-in and bounded, but adds bidirectional pair controls, same-BIO attachment/refcount subtleties, and a less uniform LibreSSL surface |
| two memory BIOs | simplest common exported mechanism; ownership and byte direction are explicit, and the pump retains every TCP error |

A BIO pair remains a reasonable later optimization if profiling shows memory-BIO
copying or allocation to be material. It would not change the public API. Its bounded
buffers are attractive, but it also needs additional header-control wrappers such as
write guarantees/read requests and careful EOF signaling, so it should not be chosen
solely to avoid enforcing progress limits in the first pump.

### 6.2 Required stream state

The safe stream needs more than a native pointer:

```siko
enum HandshakeState {
    New,
    Complete,
    Failed,
}

enum ReceiveState {
    Open,
    CloseNotifyReceived,
    TransportEof,
}

enum SendState {
    Open,
    CloseNotifySent,
}

struct TlsStream {
    tcp: TcpStream,
    ssl: NativeSsl,
    context: TlsContext,
    callbacks: CallbackRoots,
    handshake_state: HandshakeState,
    receive_state: ReceiveState,
    send_state: SendState,
    closed: Bool,
}
```

Keeping the context reachable is conservative and supports native implementations
which retain context references. Keeping callback roots reachable is mandatory.
`Failed` is terminal: after a fatal handshake/protocol failure, normal reads and
writes return the cached original failure without re-entering OpenSSL. After `close`
has run, operations return `Closed`. This prevents a second call from losing the only
useful native error queue or entering OpenSSL in an undefined state.

The Siko `TlsContext` wrapper needs a shared lease count, even though the native
`SSL_new`/`SSL_free` path has its own context lifetime rules. Each accepted stream
retains a lease containing the callback roots. Closing a listener closes its TCP
accept socket and releases only the listener's lease; it must not free callback state
still used by accepted streams. The final listener/stream lease frees the native
context and releases the roots. If streams may cross threads, this count and the
one-time free transition must use the repository's synchronization primitives.

### 6.3 The pump algorithm

Handshake, read, write, and shutdown all use one driver. In pseudocode:

```text
drive(operation):
  loop:
    clear native error queue
    result = call exact SSL operation

    if result is success:
      flush every pending byte from write BIO to TcpStream
      return operation result

    error_class = SSL_get_error(ssl, exact result)  // immediately

    if error_class is fatal:
      capture error queue now, then read the verification result

    flush every pending byte from write BIO to TcpStream

    switch error_class:
      WANT_READ:
        if read BIO still contains usable bytes:
          retry TLS operation
        else:
          read ciphertext from TcpStream
          if TCP returns bytes:
            write all of them into read BIO
            retry
          if TCP returns EOF:
            mark memory BIO EOF and retry once so SSL classifies the EOF
          if TCP returns error:
            return Io(error)

      WANT_WRITE:
        // output was flushed above
        retry

      ZERO_RETURN:
        record close_notify received
        return clean EOF for read, or the appropriate closed result

      SSL or SYSCALL:
        use the error stack and verification result captured above
        classify a bare transport EOF as UnexpectedEof
        enter terminal failure state
        return Ssl(error)

      another WANT_*:
        return Ssl(UnsupportedRetryState(...))
```

The real implementation needs a few refinements:

- It must drain the write BIO even when an SSL operation reports success. A successful
  handshake or application write may have emitted ciphertext which is still pending.
- On a fatal result it must copy the error queue immediately after `SSL_get_error`,
  before `BIO_ctrl_pending`, `BIO_read_ex`, or any other native call used to drain the
  alert. The queued error would otherwise be vulnerable to contamination or loss.
- It must drain output before blocking for more input. Peers otherwise deadlock while
  each waits for bytes held in the other's memory BIO.
- It must handle partial TCP writes by looping until all ciphertext is sent. A zero
  TCP write becomes `SslError.WriteZero`.
- It must handle a partial `BIO_write` even when feeding ciphertext, rather than
  assuming all input was accepted.
- It must not assume an application read only wants socket reads or an application
  write only wants socket writes. TLS 1.3 post-handshake messages make the opposite
  direction possible.
- It must bound temporary buffers and drain promptly. A memory BIO can grow without a
  fixed limit, so the engine should not accumulate arbitrary peer data or outbound
  ciphertext before making progress.
- It must avoid a spin when OpenSSL returns the same WANT state without consuming BIO
  input or producing output. Progress counters make this detectable as an internal
  error.

`BIO_ctrl_pending` (an exported function in the supported API) gives the number of
pending output bytes without binding the `BIO_pending` header macro.

A concrete bounding policy is to read ciphertext from TCP in one fixed 16–32 KiB
scratch slice only when OpenSSL asks for input, and feed it before reading again. A
single `SSL_write_ex` should receive at most one TLS plaintext-fragment-sized chunk
(16 KiB), flush the resulting write BIO, and return that accepted partial count; a
separate `write_all` helper can loop. Context compilation should also set a documented
maximum certificate-list size. These limits keep a caller's multi-megabyte write or a
peer's handshake from turning the memory BIO into an accidental unbounded queue.

### 6.4 Transport EOF and authenticated EOF

An empty memory BIO normally behaves like a retryable source. When `TcpStream.read`
returns zero, the engine changes the read BIO's empty behavior to EOF with
`BIO_set_mem_eof_return(rbio, 0)` (or its LibreSSL function equivalent) and re-enters
the exact SSL operation. This allows the TLS library to distinguish an authenticated
`close_notify` from a truncated record stream.

A received `close_notify` maps to `read -> Ok(0)`. A TCP EOF without `close_notify`
maps to `Ssl(UnexpectedEof)`, including when there happens to be no native error queue
entry. The implementation should not enable `SSL_OP_IGNORE_UNEXPECTED_EOF`: doing so
would turn possible truncation attacks into clean EOF.

Once `close_notify` has been received, later reads continue to return zero and do not
touch the native object. Application data received after close is a protocol error
handled by the TLS library.

### 6.5 Write semantics

The public `write` should follow Siko's ordinary partial-write semantics and return
the number of plaintext bytes OpenSSL accepted; `write_all` is a separate looping
helper. With `SSL_write_ex`, success
returns an explicit accepted length. The ciphertext corresponding to those bytes must
be completely flushed to the underlying blocking TCP stream before returning success,
otherwise a later `close` could silently discard it.

Retries must obey the OpenSSL rule that the same plaintext buffer and length are
presented after `WANT_READ`/`WANT_WRITE`, unless the exact native mode allowing moving
buffers has deliberately been enabled. The simplest safe implementation keeps the
caller's slice live and does not enable that mode.

Writing after local shutdown or peer closure returns `Ssl(Closed)`. A zero-length
application write may return `Ok(0)` without entering OpenSSL.

### 6.6 Shutdown and close

TLS shutdown and TCP close are distinct:

- `shutdown` starts or completes the TLS close-notify exchange without freeing the
  stream;
- `close` sends the local alert if possible, drains it to TCP, closes the TCP stream,
  frees native resources, and is idempotent.

For `SSL_shutdown`, return value `0` is not an SSL failure and must not be passed to
`SSL_get_error` as if it were one. It means the local `close_notify` was sent but the
peer's alert has not yet arrived. A general-purpose blocking `close` should use a
one-way fast shutdown: send and flush the local alert, then close TCP rather than wait
forever for an uncooperative peer. A separately named
`shutdown_bidirectional` could drive the receive side until the peer alert or a caller
controlled timeout.

Closing a stream still in `New` state should not start a potentially blocking
handshake merely to send an alert; it frees native state and closes TCP directly.
Explicit `shutdown` before a completed handshake returns `UnsupportedState`, leaving
the caller free to call `handshake` or `close`.

If the connection is already in a fatal SSL or syscall error state, OpenSSL advises
against calling `SSL_shutdown`. The wrapper should close TCP and free native objects
directly. Cleanup errors need a deterministic policy: return the first operational
failure, but still attempt all idempotent cleanup and retain later failures only as
diagnostic context.

## 7. Configuration-to-native behavior

### 7.1 Protocol versions

The safe config maps TLS 1.2 and 1.3 to backend constants. Invalid ranges
(`max_version < min_version`) fail before allocating a live listener. OpenSSL 3's
min/max setters are header macros over `SSL_CTX_ctrl`; the Siko macro layer implements
those calls. Current LibreSSL exposes callable min/max setters.

The default minimum should be TLS 1.2 even if a system library was built with older
protocol support. The maximum can be TLS 1.3 explicitly; using the library maximum is
reasonable only while the public `Version` and connection-state mapping reject
unknown future versions cleanly.

### 7.2 Cipher suites

OpenSSL separates TLS 1.2-and-earlier cipher configuration
(`SSL_CTX_set_cipher_list`) from TLS 1.3 ciphersuites
(`SSL_CTX_set_ciphersuites`). The Go-like field should call only the former after
mapping IANA IDs to OpenSSL names, because the field intentionally does not override
TLS 1.3 defaults.

LibreSSL's `SSL_CTX_set_cipher_list` documentation includes TLS 1.3 configuration in
the same cipher expression. The LibreSSL adapter must preserve the public contract:
custom TLS 1.2 choices must not accidentally disable or weaken the backend's TLS 1.3
defaults. Current LibreSSL documents the needed rule: when a non-empty control string
selects suites but neither mentions `TLSv1.3` nor explicitly includes/excludes a TLS
1.3 suite, all LibreSSL TLS 1.3 suites remain available. The adapter should therefore
emit only the mapped TLS 1.2 names and reject any raw token which could affect TLS 1.3.
Integration tests must still negotiate both versions so a future supported LibreSSL
branch cannot silently change that assumption.

Raw OpenSSL cipher strings can be offered later in a backend-specific expert config,
not in the portable safe API.

### 7.3 Server certificates and client authentication

During server-context compilation:

1. parse and install the leaf certificate;
2. install each intermediate in order;
3. parse and install the private key;
4. call the native private-key consistency check;
5. fail listener construction if any step fails.

PEM parsing needs careful queue handling. Repeatedly calling a PEM reader until it
fails can leave an expected “no start line” error in the queue and contaminate a later
operation. The implementation should either parse PEM blocks in Siko and feed exact
DER objects, or use BIO position/pending information to distinguish expected end of
input and clear that terminal parse result deliberately.

The client-auth modes map conceptually as follows:

| Siko mode | Request a certificate | Require one | Verify it against `client_cas` |
|---|---:|---:|---:|
| `NoClientCert` | no | no | no |
| `RequestClientCert` | yes | no | no |
| `RequireAnyClientCert` | yes | yes | no |
| `VerifyClientCertIfGiven` | yes | no | yes, if supplied |
| `RequireAndVerifyClientCert` | yes | yes | yes |

The backend implements those semantics with the documented `SSL_VERIFY_*` flags and
a trust store. `VerifyClientCertIfGiven` and `RequireAndVerifyClientCert` require a
valid `client_cas` policy. The certificate-authority names advertised to clients are a
separate native list and should be built from that pool where supported; merely
setting a verification store is not always enough to populate the handshake hint.
OpenSSL can add each configured CA certificate with exported
`SSL_CTX_add_client_CA`, avoiding typed stack macros in this path.

Flags alone do not implement the two “accept any certificate” modes. In OpenSSL and
low-level LibreSSL, `SSL_VERIFY_PEER` makes the server request a certificate and the
default verification callback aborts on an invalid chain. `RequestClientCert` and
`RequireAnyClientCert` therefore install a tiny direct `@c_callback` which always
continues verification; the latter additionally sets
`SSL_VERIFY_FAIL_IF_NO_PEER_CERT`. The callback deliberately accepts an untrusted
chain, but the raw peer chain is still copied into `ConnectionState` and
`verified_chains` remains empty. The two verified modes use the default callback so a
bad chain terminates the handshake. This is one more core callback, but it needs no
application state and no C shim.

LibreSSL `libtls` directly documents only required and optional *verified* client
certificate modes. It does not expose the low-level verification callback needed to
promise Go's two unverified modes independently. The 4.3 `libtls` backend therefore
rejects `RequestClientCert` and `RequireAnyClientCert` as unsupported configuration
rather than silently verifying them or silently weakening a different mode.

### 7.4 Client verification and SNI

Verification has three separate parts which must all be configured:

1. enable peer certificate-chain verification;
2. install custom or system trust roots;
3. configure hostname or IP verification for the intended server identity.

On OpenSSL 3, `SSL_set1_host` configures the expected host/IP for certificate
verification. DNS SNI is configured separately with the header macro
`SSL_set_tlsext_host_name`; setting one is not a substitute for the other. The
LibreSSL adapter should apply the equivalent pair.

No custom “check the certificate common name” code should be written in Siko. Native
hostname validation correctly handles subject alternative names, wildcards, IP SANs,
and name constraints. The common-name fallback policy should follow the selected
library and be documented if it differs.

`insecure_skip_verify` skips both chain and identity checks. It must not silently
change the SNI name: SNI can still be sent for server certificate selection unless
the configured identity is an IP literal.

The safe server-name parser should accept an IP literal or an ASCII DNS A-label, not
a URL, `host:port`, or bracketed address. It rejects embedded NUL and malformed/empty
labels, canonicalizes an absolute DNS name's trailing dot consistently, and sends no
SNI for an IP. Until Siko has a reviewed IDNA implementation, callers supply Unicode
names in A-label (punycode) form; passing raw Unicode must not produce a locale- or
backend-dependent C string.

### 7.5 ALPN

`next_protocols` is encoded in the standard ALPN wire format: one length byte followed
by 1–255 protocol bytes for each entry. Empty entries and entries longer than 255
bytes are configuration errors; duplicate entries should be rejected or normalized.
The whole encoded vector must fit the native length type.

The OpenSSL client call `SSL_set_alpn_protos` has an unusual convention: zero means
success. The binding wrapper should convert it to an ordinary Siko `Result` so that
call sites cannot accidentally invert it.

The server selection callback is a direct `@c_callback`. It should use server
preference order, validate the untrusted client wire vector, and point the selected
output at bytes in the client-offered buffer for only the lifetime guaranteed by the
API. The callback's Siko policy state remains rooted by `TlsContext`. To follow Go's
behavior, a peer which sends an ALPN extension with no mutually supported protocol
causes handshake failure, while a peer which sends no ALPN extension can connect with
an empty negotiated protocol.

### 7.6 Sessions and post-handshake behavior

A portable configurable session cache and ticket-key rotation are later features.
`did_resume` is still safe to expose in `ConnectionState` when the backend provides
it.

With the proposed per-stream client context, version-one clients effectively do not
resume across connections, matching Go when no client session cache is configured.
A listener's shared server context may use the backend's default ticket/session
behavior; its ticket keys live with that context and disappear when the final context
lease is released. A future reusable `TlsConnector` and explicit ticket policy should
make cross-connection resumption a deliberate, testable contract.

TLS 1.3 post-handshake messages are why every SSL operation must honor both WANT
directions. Post-handshake client authentication should be out of scope initially.
Renegotiation should remain disabled where applicable; TLS 1.3 does not use the old
renegotiation mechanism.

## 8. Installed OpenSSL 3 backend

### 8.1 PoC environment and runtime guard

The PoC must use what is already installed on this laptop; installing, downgrading,
or adding a second SSL distribution is out of scope. The inspected environment is
macOS arm64 with Homebrew OpenSSL 3.6.2. `pkg-config openssl` resolves its headers and
libraries, and the stable Homebrew prefix is `/opt/homebrew/opt/openssl@3`. The
required headers, shared/static libraries, memory-BIO symbols, `SSL_read_ex`,
`SSL_write_ex`, `SSL_get_error`, and error-queue functions are present.

The version is an observed implementation input, not an API requirement invented by
the TLS design. The PoC should audit and translate the installed 3.6.2 headers. A
later portability policy can define other tested OpenSSL 3 releases from actual build
requirements and CI evidence.

The backend package should:

- obtain the existing include/library directories from `pkg-config openssl`, with
  `/opt/homebrew/opt/openssl@3` as the local stable fallback;
- call the widely compatible `OpenSSL_version(OPENSSL_VERSION)` first;
- require an identity string beginning with `OpenSSL 3.` so the runtime matches the
  selected OpenSSL 3 headers;
- reject `LibreSSL ...`, OpenSSL 1.x, and OpenSSL 4.x before using version-specific
  functions or constants;
- include the runtime identity in an `InvalidConfiguration` error to make loader-path
  problems diagnosable.

OpenSSL 4 can be a future backend binding set. It should not be assumed compatible
just because most high-level function names remain familiar.

### 8.2 Raw and safe binding layers

A useful internal separation is:

```text
Std.Net.Tls.Backend.OpenSsl3.Raw
  exact exported C functions, primitive constants, void* handles

Std.Net.Tls.Backend.OpenSsl3.Macros
  Siko translations of selected header macros

Std.Net.Tls.Backend.OpenSsl3.Error
  error-queue capture and classification

Std.Net.Tls.Backend.OpenSsl3.Context
  safe SSL_CTX construction and configuration

Std.Net.Tls.Backend.OpenSsl3.Connection
  safe SSL/BIO ownership and TLS operations

Std.Net.Tls.Engine
  backend-neutral TCP/memory-BIO pump where practical
```

Raw extern declarations can take `void*` for opaque C structures, wrapped immediately
in distinct private Siko handle structs to restore type safety. No opaque C structure
layout should be reproduced. Siko must never dereference an `SSL`, `SSL_CTX`, `BIO`,
`X509`, or `EVP_PKEY` as if its fields were public.

The bindings should use checked conversions for C `int`, `long`, `unsigned long`,
`size_t`, and Siko lengths. The first target is 64-bit Darwin/Linux LP64; every public
buffer length still needs a range check before narrowing to C `int` APIs. Prefer the
`*_ex` read/write APIs with `size_t` lengths where available.

Raw externs should use ABI integers and pointers only: C `int`/enum/boolean results
become `I32`, not Siko `Bool`; C strings become `U8*`/`I8*`, not Siko `String`; and
buffers are pointer-plus-C-length pairs, not Siko `Slice` passed by layout. The safe
layer alone converts `0`/`1`, builds checked C strings, pins/roots buffers for the call,
and turns output lengths into Siko `Int`.

### 8.3 Direct exported functions needed for the core

The exact list should be generated/audited against the installed OpenSSL 3.6.2 headers,
but the core categories are:

- initialization and identity: `OpenSSL_version`, and optional explicit initialization
  if the supported library requires configuration flags;
- contexts: `TLS_method`, `SSL_CTX_new`, `SSL_CTX_free`;
- connections: `SSL_new`, `SSL_free`, `SSL_set_connect_state`,
  `SSL_set_accept_state`, `SSL_do_handshake`, `SSL_get_error`, `SSL_shutdown`;
- plaintext I/O: `SSL_read_ex`, `SSL_write_ex`;
- BIOs: `BIO_s_mem`, `BIO_new`, `BIO_free`, `BIO_read_ex`, `BIO_write_ex`,
  `BIO_ctrl`, `BIO_ctrl_pending`, and `BIO_new_mem_buf` for parsing;
- verification and trust: `SSL_CTX_set_verify`,
  `SSL_CTX_set_default_verify_paths`, `SSL_CTX_load_verify_locations`,
  `SSL_CTX_get_cert_store`, `X509_STORE_add_cert`, `SSL_set1_host`,
  `SSL_CTX_add_client_CA`, `SSL_get0_param`, IP verification-parameter setters,
  `SSL_get_verify_result`, and `X509_verify_cert_error_string`;
- verification callback support: the exact `SSL_CTX_set_verify` callback ABI for the
  Go-like unverified client-certificate modes;
- certificates and keys: PEM/DER parse functions, `SSL_CTX_use_certificate`,
  `SSL_CTX_use_PrivateKey`, `SSL_CTX_check_private_key`, `X509_check_private_key`,
  `X509_free`,
  `EVP_PKEY_free`;
- ALPN: `SSL_set_alpn_protos`, `SSL_CTX_set_alpn_select_cb`,
  `SSL_get0_alpn_selected`;
- cipher configuration and state inspection: `SSL_CTX_set_cipher_list`, protocol,
  cipher, session, peer-chain, and SNI accessors actually used by
  `ConnectionState`;
- copied chain inspection: `OPENSSL_sk_num`, `OPENSSL_sk_value`, and X509 DER
  serialization for borrowed peer/verified-chain objects;
- diagnostics: `ERR_clear_error`, `ERR_get_error_all`, `ERR_error_string_n`, and
  available library/reason string accessors.

Only functions exercised by the safe layer should be bound. A giant unreviewed
translation of all OpenSSL headers would enlarge the audit and versioning burden.

### 8.4 Header macros to implement in Siko

OpenSSL's public headers expose some operations as macros over stable exported control
functions. Declaring these macro names with `@extern` would compile but fail at link
time because no function symbol exists. The required wrappers are small Siko
functions:

| Public header operation | Siko implementation strategy |
|---|---|
| `SSL_CTX_set_min_proto_version` | call `SSL_CTX_ctrl` with `SSL_CTRL_SET_MIN_PROTO_VERSION` |
| `SSL_CTX_set_max_proto_version` | call `SSL_CTX_ctrl` with `SSL_CTRL_SET_MAX_PROTO_VERSION` |
| `SSL_set_tlsext_host_name` | call `SSL_ctrl` with `SSL_CTRL_SET_TLSEXT_HOSTNAME` and `TLSEXT_NAMETYPE_host_name` |
| `BIO_set_mem_eof_return` | call `BIO_ctrl` with `BIO_C_SET_BUF_MEM_EOF_RETURN` |
| `SSL_CTX_set_max_cert_list` | call `SSL_CTX_ctrl` with `SSL_CTRL_SET_MAX_CERT_LIST` |
| `SSL_CTX_add1_chain_cert` | call `SSL_CTX_ctrl` with `SSL_CTRL_CHAIN_CERT` and the documented ownership flag |
| server-name callback setter | call `SSL_CTX_callback_ctrl` with `SSL_CTRL_SET_TLSEXT_SERVERNAME_CB` |
| server-name callback argument setter | call `SSL_CTX_ctrl` with `SSL_CTRL_SET_TLSEXT_SERVERNAME_ARG` |

The wrappers must use the exact casts and return-value tests from the installed
OpenSSL 3.6.2 header definitions. Function-pointer casts for
`SSL_CTX_callback_ctrl` are an `@unsafe` implementation detail.

In Siko-like pseudocode, the ordinary control macros are no more than these direct
calls (the raw `long` type is an LP64-sized private binding type):

```siko
fn ssl_ctx_set_min_proto_version(ctx: void*, version: I32) -> Bool {
    SSL_CTX_ctrl(ctx, SSL_CTRL_SET_MIN_PROTO_VERSION, version.to_c_long(), null) > 0
}

fn ssl_set_tlsext_host_name(ssl: void*, name: CString) -> Bool {
    SSL_ctrl(
        ssl,
        SSL_CTRL_SET_TLSEXT_HOSTNAME,
        TLSEXT_NAMETYPE_host_name.to_c_long(),
        name.ptr().as_void()
    ) > 0
}

fn bio_set_mem_eof_return(bio: void*, value: I32) -> Bool {
    BIO_ctrl(bio, BIO_C_SET_BUF_MEM_EOF_RETURN, value.to_c_long(), null) > 0
}
```

The actual code must mirror the selected header's parenthesization, pointer casts,
and exact success convention. Callback-control and chain-certificate wrappers follow
the same principle but deserve separate unsafe functions because they transfer a
function pointer or reference-counted object.

Constants should live in one versioned module, copied or generated from the installed
OpenSSL 3.6.2 headers with a provenance comment. Do not scatter numeric control codes
through the backend. A development-time binding generator may preprocess headers and
commit Siko output; this is still a direct-FFI runtime and does not introduce a C shim
or require compiling C in user builds.

Several macro families should be avoided rather than translated:

- use `BIO_ctrl_pending` instead of `BIO_pending`;
- use exported error formatting/accessor functions rather than `ERR_GET_*` bit macros;
- add chain certificates individually rather than depending on `STACK_OF(X509)` type
  construction macros;
- use exported opaque-structure accessors rather than any field-access macro;
- omit a feature if its only implementation requires private structure layout.

Read-only peer and verified-chain accessors do return `STACK_OF(X509)`. The raw layer
should bind only exported `OPENSSL_sk_num` and `OPENSSL_sk_value`, wrap their casts in
private typed Siko helpers, and copy each borrowed certificate to DER before another
native operation. It does not construct or mutate these stacks. This limited use is
safer than reproducing OpenSSL's typed stack macro family and is needed for the
proposed `ConnectionState`.

### 8.5 Provider and FIPS considerations

OpenSSL 3 introduces providers. The first backend should rely on normal OpenSSL
configuration and default-provider loading, avoid deprecated ENGINE APIs, and report
provider-related failures through the ordinary error stack. Explicit library
contexts, provider selection, property queries, and FIPS-mode policy are a later
backend-specific extension; they should not leak into the portable `Config` until
there is a concrete supported use case.

## 9. LibreSSL alternatives

### 9.1 Version and support policy

As of this study, LibreSSL 4.3.2 is the current stable release. LibreSSL releases on a
roughly six-month cadence and supports branches for a much shorter period than an
OpenSSL LTS. A Siko backend should consequently name and test an explicit supported
LibreSSL branch rather than promise a long ABI range.

`OpenSSL_version_num` is deliberately a compatibility value in LibreSSL and cannot
identify the real LibreSSL release. The backend should first call
`OpenSSL_version(OPENSSL_VERSION)`, require a `LibreSSL ...` identity string, and parse
the release text for diagnostics/policy. It must reject an OpenSSL identity before
using LibreSSL-specific symbols.

LibreSSL supports TLS 1.3 in current releases. Some individual manual pages retain
older wording which lists protocol constants only through TLS 1.2, while current
release notes and connection/cipher documentation include TLS 1.3. The implementation
should use the selected release's headers/source as the binding authority and enforce
TLS 1.2/TLS 1.3 negotiation in CI, rather than copying stale prose into the public
capability model.

### 9.2 Low-level LibreSSL `libssl`

This is the closest alternative to the OpenSSL design:

- allocate `SSL_CTX` and per-connection `SSL` objects;
- attach two memory BIOs;
- drive handshake/read/write/shutdown with the same state machine;
- use the same `TlsStream`, `TlsListener`, `Config`, and top-level error shape;
- isolate native error capture and configuration differences behind the backend.

The core APIs for `SSL_get_error`, memory BIOs, certificate verification, hostname
checking, ALPN, SNI, and TLS I/O are sufficiently similar for this architecture.
Important differences include:

- LibreSSL uses the older error-queue functions rather than OpenSSL 3's
  `ERR_get_error_all`;
- several operations which are OpenSSL header macros are exported LibreSSL functions,
  including current min/max protocol setters and `BIO_set_mem_eof_return`;
- SNI convenience operations remain macros and still need Siko translations;
- certificate-chain helpers and cipher-list behavior differ;
- OpenSSL provider APIs do not apply;
- return conventions and ownership must be checked against the chosen LibreSSL
  release, not inferred from matching names.

The backend should have its own raw and macro modules rather than sharing unchecked
extern declarations with OpenSSL. The higher-level pump and public API can share code
where their internal backend trait can express ownership and error differences
without erasing them.

Because both libraries export `SSL_*`, `BIO_*`, and `ERR_*` symbols from libraries
usually named `ssl` and `crypto`, backend choice is made at compile/package selection.
Trying to support both in one executable through ordinary linking risks symbol
interposition and undefined behavior.

### 9.3 High-level LibreSSL `libtls`

LibreSSL also ships `libtls`, a deliberately smaller TLS API. A Siko backend would
link `tls`, `ssl`, and `crypto`, then use:

- `tls_config_new`/`tls_config_free` and configuration setters;
- `tls_client` or `tls_server`, followed by `tls_configure`;
- `tls_connect_cbs` for a client over an existing `TcpStream`;
- `tls_accept_cbs` for accepted server connections;
- `tls_handshake`, `tls_read`, `tls_write`, and `tls_close`;
- `tls_error` and the connection-inspection functions.

The callback forms are decisive. They accept C read/write function pointers and an
opaque state pointer, so Siko can provide direct `@c_callback` functions which invoke
the existing TCP stream. This satisfies the no-C-shim requirement and avoids both
`SSL_set_fd` and custom OpenSSL BIO construction.

A callback bridge would look conceptually like:

```siko
struct TransportCallbackState {
    stream: TcpStream,
    last_io_error: Option[IO.Error],
    write_zero: Bool,
}

@unsafe
@c_callback
fn tls_read_callback(state: void*, out: void*, len: U64) -> I64 {
    // Recover rooted state, call TcpStream.read, store exact error on failure,
    // return bytes/EOF/-1 according to libtls's callback ABI.
}
```

The state must be reachable from the Siko `TlsStream`, not just from `void*` held by
libtls. The outer call clears `last_io_error` and `write_zero`; after a native failure
it returns the stored transport problem first, otherwise copies `tls_error` before
the next libtls call.

`libtls` reports `TLS_WANT_POLLIN` and `TLS_WANT_POLLOUT` for retryable nonblocking
states. The first Siko backend remains blocking, but should classify these internally
so a future event-loop version is not precluded.

#### Benefits of `libtls`

- much smaller FFI and ownership surface;
- no need to reproduce the OpenSSL SSL/BIO control macros used by the low-level
  engine;
- secure configuration defaults and simple in-memory CA/certificate/key setters;
- callback transport integration with the existing Siko TCP API;
- simpler ALPN, protocol, and connection-inspection operations.

#### Costs of `libtls`

- `tls_error` primarily supplies one current error string, not a structured
  `SSL_get_error` classification plus a complete native error queue;
- exact I/O errors require the out-of-band callback-state technique above;
- fine-grained certificate selection, verification callbacks, session behavior, and
  other advanced Go-like hooks are less flexible or unavailable;
- the two Go modes which request/require a client certificate without verifying its
  chain are not part of the documented high-level `libtls` client-auth model and
  are rejected by this backend;
- `libtls` configures ALPN through a comma-delimited C string rather than the raw wire
  vector, so that backend must reject an otherwise valid protocol identifier
  containing a comma (and, like all backends, embedded NUL);
- its high-level policy may change observable details relative to the low-level
  OpenSSL backend;
- async callback integration is not automatically solved: a C callback cannot be
  assumed to suspend a Siko coroutine safely across FFI;
- it adds another native library (`libtls`) and is specific to LibreSSL.

The public top-level error enum can remain identical, but `SslLibraryError.stack` will
usually contain one synthesized entry holding the copied `tls_error` string. Code
must not promise OpenSSL error codes when running this backend.

### 9.4 Backend comparison

| Concern | Installed OpenSSL 3.6.2 `libssl` | LibreSSL `libssl` | LibreSSL `libtls` |
|---|---|---|---|
| Existing `TcpStream` used for all I/O | yes, memory BIO pump | yes, memory BIO pump | yes, read/write callbacks |
| C shim required | no | no | no |
| Siko callbacks needed in core | server ALPN; later SNI selection | server ALPN; later SNI selection | transport callbacks, plus policy callbacks as available |
| FFI surface | largest | similarly large | smallest |
| OpenSSL-style macro work | several required wrappers | fewer, but SNI still macro-like | essentially avoided at public libtls level |
| Detailed native error stack | strongest | available, older API | weak/single string |
| Exact Siko `IO.Error` | direct from pump | direct from pump | callback side channel |
| Public Go-like config parity | strongest | good with adapter differences | core subset only |
| TLS 1.2 and 1.3 | yes | yes | yes |
| Available for this PoC without installation | yes | no | no |
| Advanced future hooks | broadest | broad | intentionally limited |
| Recommended role | primary backend | parity alternative | simpler reduced-scope alternative |

The PoC recommendation is the already-installed OpenSSL 3.6.2 low-level library. A
LibreSSL backend remains a valid design alternative, but implementing or testing it
on this laptop would violate the no-install constraint because LibreSSL is not
present. If it is evaluated later in an environment where it already exists, use
low-level `libssl` for parity or `libtls` as a deliberately smaller backend with
reduced error detail.

## 10. Ownership, concurrency, and API edge cases

### 10.1 Ownership transfer in a non-linear type system

The public API should document logical ownership even though current Siko values do
not enforce a Rust-like move:

- after `TlsStream.client(tcp, config)` or `server` succeeds, the TLS stream is the
  sole logical owner and the caller must not use or close its old TCP alias;
- after `TlsListener.from_tcp_listener` succeeds, the same rule applies to the
  listener;
- if either wrapping constructor fails, it frees native objects it allocated but
  leaves the caller-provided TCP object open;
- if the `connect` or `bind` convenience allocates TCP internally and later TLS setup
  fails, it closes that TCP resource before returning the error.

This failure rule is both predictable and implementable. A future ownership-aware
language feature could enforce it, but TLS should not wait for one.

There should initially be no `get_ssl`, `get_bio`, `get_tcp_stream`, or
`into_tcp_stream` escape hatch. Exposing the file descriptor read-only would still
let callers bypass encryption or close it underneath TLS. If an event loop later
needs readiness information, expose backend-neutral readiness through the async TLS
type rather than native handles on the safe blocking type.

### 10.2 Concurrent operations

The memory-BIO pump mutates shared read BIO, write BIO, handshake, shutdown, and error
queue state. It is not safe for two Siko threads to call it concurrently without
serialization. Version one should explicitly require that operations on a particular
`TlsStream` are not concurrent. `TlsContext` can be shared for constructing separate
connections, and a listener can hand separate streams to separate threads.

If matching Go's allowance for concurrent methods becomes important, add internal
state and I/O locks only after testing the interaction among a read that needs to
write, a write that needs to read, and close. A single coarse connection lock is
correct but removes useful full-duplex behavior; two naive direction locks can
deadlock because TLS operations cross directions. This is a real engine feature, not
an API-documentation tweak.

### 10.3 Deadlines and cancellation

The current base TCP API does not expose Go-style deadlines. TLS should not add
configuration fields which it cannot enforce on the underlying blocking read and
write. Handshake timeouts belong in a future deadline/cancellation facility shared
with TCP or in the async implementation. Lazy listener handshakes at least keep a
slow peer from blocking the `accept` operation itself.

### 10.4 Initialization and process-global state

Modern OpenSSL normally initializes itself automatically, but configuration/provider
loading can still have process-wide consequences. The backend should use a one-time,
thread-safe initialization path, avoid global cleanup calls while any TLS value may
exist, and never mutate process-global allocator hooks. LibreSSL initialization should
follow the selected branch's documented requirements.

Because `SSL_get_error` depends on a thread-local queue, the clear/call/classify/drain
sequence is one indivisible native operation. A `TlsStream` operation must not migrate
to a different OS thread in the middle of that sequence. The current synchronous
call stack naturally satisfies that; a future coroutine implementation must preserve
it or perform the whole native step before yielding.

## 11. Package and source layout

A clean split would be:

```text
std/TlsShared/                    # repository source input, not a selectable backend
  Common/
    Net/Tls.sk
    Net/Tls/Error.sk
    Net/Tls/Certificate.sk
    Net/Tls/Config.sk
    Net/Tls/Engine.sk

std/TlsOpenSsl3/
  package.toml
  Public/
    Net/TlsStream.sk
    Net/TlsListener.sk
  OpenSsl3/
    Raw.sk
    Constants.sk
    Macros.sk
    Error.sk
    Context.sk
    Connection.sk

std/TlsLibreSsl/
  package.toml
  Public/
    Net/TlsStream.sk
    Net/TlsListener.sk
  LibreSsl/
    Raw.sk
    Constants.sk
    Macros.sk
    Error.sk
    Context.sk
    Connection.sk

std/TlsLibreTls/                 # optional alternative, not required for parity
  package.toml
  Public/
    Net/TlsStream.sk
    Net/TlsListener.sk
  LibreTls/
    Raw.sk
    Callback.sk
    Context.sk
    Connection.sk
```

The exact module/package mechanics should follow current repository package source,
not old documentation. The key properties are:

- common public value types contain no backend pointers;
- exactly one backend supplies the private context/connection implementation to a
  program;
- OpenSSL's package declares native `ssl` and `crypto` candidates;
- LibreSSL low-level declares its own `ssl` and `crypto` installation candidates;
- `libtls` additionally declares `tls`;
- native library discovery returns a coherent directory containing one selected
  distribution, so the linker and rpath cannot mix `libssl` from one installation
  with `libcrypto` from another;
- ordinary `Std` programs do not acquire a TLS native dependency.

The current package manifest/parser has dependencies and native libraries but no
feature or mutually exclusive provider mechanism. Backend selection is therefore the
dependency package itself: `StdTlsOpenSsl3` and `StdTlsLibreSsl` can expose mechanically
aligned `Std.Net.Tls*` modules, but an application includes exactly one of them. A
third `StdTlsLibreTls` exposes the documented reduced subset. Pulling two backends
into one dependency graph is a package error, not a runtime selection. Shared public
declarations/engine code can be maintained as repository source input or generated
Siko source until the package system grows a provider abstraction. Runtime `dlopen`
is not the solution: direct linking gives earlier failures, simpler FFI, and less
symbol ambiguity.

Native-library commands should return an installation's library directory in the
same manner as current package-native candidates. Header location is only a binding
development concern because committed direct FFI declarations do not compile headers
in user builds.

The current native schema resolves each library independently and checks only that a
candidate result is a directory. It does not verify that `libssl` and `libcrypto`
both exist there or that they came from one distribution. The package should put
`ssl` before `crypto` in the `libs` array for static/link-order compatibility and give
both entries the exact same ordered candidate list, for example conceptually:

```toml
[native]
libs = [
  { name = "ssl", candidates = [
    { command = "pkg-config", args = ["--variable=libdir", "openssl"] },
    { path = "/opt/homebrew/opt/openssl@3/lib" },
  ] },
  { name = "crypto", candidates = [
    { command = "pkg-config", args = ["--variable=libdir", "openssl"] },
    { path = "/opt/homebrew/opt/openssl@3/lib" },
  ] },
]
```

The Homebrew fallback is deliberately local to this PoC; it is not a universal
deployment path. A future package-level “native bundle” candidate would make atomic
multi-library selection safer. Until then the identical candidate lists, an explicit
backend package, the runtime identity guard, and CI inspection of the resolved binary
are all necessary. LibreSSL packages use their own locator/module and never reuse the
OpenSSL fallback list.

The initial packaging promise should be shared-library linking. A static OpenSSL or
LibreSSL build can require additional transitive system libraries and linker flags
which the current directory-plus-`-l<name>` native schema cannot obtain from the full
`pkg-config --libs --static` output. Static support should wait for either explicit
tested extra native entries or a package feature for complete linker-argument sets.

## 12. Implementation sequence

### Phase 0: lock the portable contract

- Add backend-neutral `Version`, `Config`, `Certificate`, `CertPool`, `Error`, and
  state types.
- Make `Tls.Error` implement the standard `Error` trait and define direct
  `Std.IO.Reader`, `Writer`, and `Closer` conformance for `TlsStream`.
- Define constructor ownership and close behavior.
- Define the TLS 1.2 cipher-suite ID set common to both initial backends.
- Keep advanced callbacks, session configuration, system-native trust stores, and
  async behavior out of the first contract.

### Phase 1: audited OpenSSL raw layer

- Use the already-installed OpenSSL 3.6.2 headers as the binding source; do not
  install or select another native package for the PoC.
- Add only the required exported functions.
- Translate required macros into Siko with named constants and provenance.
- Add native identity rejection before using the selected OpenSSL 3 APIs.
- Implement error-queue capture independently of networking.
- Add failure-atomic wrappers for `SSL_CTX`, `SSL`, `BIO`, `X509`, and `EVP_PKEY`.

### Phase 2: config and certificates

- Implement PEM/DER parsing and key-match validation.
- Compile protocol, ciphers, trust, server certificate, and client-auth policy into a
  context.
- Implement DNS/IP identity verification and DNS-only SNI.
- Implement ALPN encoding and the server callback with rooted state.
- Make all invalid config fail before the listener begins accepting.

For LibreSSL specifically, DNS names can use `SSL_set1_host`; current LibreSSL
documentation describes that API as DNS-hostname validation only. IP literals should
use `X509_VERIFY_PARAM_set1_ip_asc` on the parameters obtained with `SSL_get0_param`,
and still omit SNI. This is a backend difference hidden behind one portable
identity-setting operation.

### Phase 3: blocking connection engine

- Attach memory BIOs and set client/server handshake state.
- Implement exact error-queue discipline.
- Implement output draining and input feeding with partial-operation handling.
- Implement lazy/explicit handshake, plaintext read/write, clean EOF, truncated EOF,
  one-way shutdown, and idempotent close.
- Add progress checks and bounded temporary buffers.

### Phase 4: listener and client convenience

- Compile and share one immutable server context in `TlsListener`.
- Implement lazy `accept` and explicit `accept_and_handshake`.
- Implement `TlsStream.connect` over the current IPv4-literal TCP connect.
- Verify TLS-backed use of the standard `Error`-returning network effect.
- Copy connection state into backend-neutral values.

### Phase 5: LibreSSL backend

- Implement either low-level `libssl` for behavioral parity or `libtls` for the
  explicitly reduced feature set.
- Run the same portable behavioral suite against it in a separate build job.
- Add backend-specific tests for error detail, cipher expression behavior, version
  identity, callbacks, and native ownership.

### Phase 6: later work

- an async TLS stream whose engine yields explicit read/write readiness;
- reusable client contexts/session caches;
- multiple certificates and SNI selection;
- OS-native trust integration where OpenSSL paths are insufficient;
- configurable curves/signature algorithms only when portable semantics are clear;
- safe deadline/cancellation support shared with TCP.

## 13. Test strategy

No implementation should be considered complete merely because it can connect to one
public HTTPS server. The tests need to target state-machine edges and ownership.

### 13.1 Pure Siko tests

- default config has verification on and a TLS 1.2 minimum;
- invalid version ranges fail;
- empty, over-255-byte, duplicate, embedded-NUL, and oversized ALPN inputs are handled
  according to the declared policy;
- server names and every C-string configuration value reject embedded NUL;
- cipher-suite IDs map deterministically and unknown IDs fail;
- TCP `IO.Error` values retain their code/message inside `Tls.Error.Io`;
- every TLS operation returns an `Error` object containing the complete
  `Tls.Error`, without flattening it to a message;
- `error is Tls.Error` narrows successfully and the resulting `Io`/`Ssl` match is
  exhaustive;
- unrelated concrete errors remain distinguishable at the same open effect
  boundary;
- `SslError` formatting preserves operation, classification, and stack order;
- all length conversions reject overflow rather than truncate.

### 13.2 Macro and ABI tests

No C shim is needed even for validation. Behavioral tests can verify the translated
macros through exported getters or a real handshake:

- min/max protocol wrappers actually reject versions outside the range;
- SNI set through the translated control macro is observed by a server callback;
- memory BIO EOF behavior produces the expected TLS truncation classification;
- chain-certificate addition sends intermediates and has correct reference ownership;
- repeated context/connection construction and cleanup detects double frees under a
  native sanitizer build where available;
- callback function-pointer signatures work under both Darwin and Linux ABI jobs;
- runtime version checks reject the wrong OpenSSL major and reject LibreSSL in the
  OpenSSL backend (and vice versa).

The constant module should be regenerated/diffed when the supported native header
revision changes. Behavioral ABI tests remain necessary because copied numeric
constants can compile cleanly while doing the wrong control operation.

### 13.3 Deterministic transport tests

The pump should depend internally on a tiny private transport interface implemented
by `TcpStream` and by a test transport. This does not make the public API generic; it
allows deterministic tests for:

- one-byte reads and writes;
- every possible short-write boundary;
- `EAGAIN` surfacing in the unsupported nonblocking v1 path;
- read and write errors before, during, and after handshake;
- transport EOF at every TLS-record boundary;
- zero underlying write;
- a peer that accepts input but never produces progress;
- outbound data pending when the native operation reports success or failure;
- a read requiring a write and a write requiring a read;
- no busy loop on repeated WANT states.

If an internal transport interface is undesirable, a local fault-injection proxy can
exercise most cases, but it will be slower and less deterministic. The production
implementation still delegates actual ciphertext I/O to the existing `TcpStream`.

### 13.4 Client/server integration tests

Use repository-owned certificate fixtures with fixed validity windows suitable for
test time control. At minimum test:

- TLS 1.2 and TLS 1.3 success;
- RSA and ECDSA server certificates if both are promised;
- a complete intermediate chain;
- unknown CA, missing intermediate, expired, not-yet-valid, wrong key usage, and
  certificate/private-key mismatch;
- DNS SAN success, DNS mismatch, wildcard boundaries, IP SAN success, and an IP
  mismatch;
- verified client certificates for every `ClientAuthType` row;
- no client certificate where optional and where required;
- ALPN selection in server preference order and no-overlap failure;
- large application payloads, zero-length buffers, partial application reads, and
  alternating full-duplex traffic;
- explicit handshake, lazy handshake through first read/write, and handshake failure
  from either path;
- clean `close_notify`, peer one-way shutdown, TCP truncation with a complete record,
  and TCP truncation inside a record;
- repeated `close`, close after construction failure, and close after fatal handshake;
- an accepted stream continuing safely after its listener is closed, including ALPN
  or verification callback state owned by the shared context;
- `ConnectionState` values and the lifetime of copied state after the stream closes;
- listener accept returning before a slow client's TLS handshake completes.

### 13.5 Native interoperability tests

Separate CI jobs should test against the supported native installation, never both
backends in one process:

- Siko client to native OpenSSL/LibreSSL server tool;
- native client tool to Siko server;
- protocol/cipher mismatch failures;
- SNI certificate routing once multiple certificates are implemented;
- ALPN with a standard protocol identifier;
- session resumption only when it becomes a supported promise.

The normal suite must not require internet access. Loopback peers and committed test
fixtures make failures reproducible.

### 13.6 Callback and GC stress tests

- force collections between context creation and every callback invocation;
- make the original config value unreachable while the compiled context stays live;
- negotiate ALPN repeatedly while allocating/growing unrelated vectors;
- create and close many listener connections to detect retained callback roots;
- for `libtls`, inject read/write errors and confirm the exact stored `IO.Error` wins
  over the generic native string;
- ensure callback failures never unwind across C and always return the documented
  native code.

### 13.7 Security regression tests

- stale OpenSSL queue entries never appear in a later error;
- unexpected EOF never becomes `Ok(0)`;
- `insecure_skip_verify = false` cannot construct a client without a verification
  identity unless a deliberately different verification policy is added later;
- system-root loading failure is not treated as an empty successful trust store;
- malformed PEM/DER and malformed ALPN never read out of bounds;
- untrusted peer lengths cannot overflow Siko/native conversions;
- callback output never points into a Siko vector which can reallocate;
- no error message includes private-key bytes or arbitrary unbounded peer data.

## 14. Security and operational limits

The initial documentation should state these limits plainly:

- TLS authenticates the configured peer identity; it does not resolve hostnames or
  decide application protocols.
- Certificate revocation is not checked unless an explicit CRL/OCSP policy is later
  implemented. Native chain validation alone is not a promise of online revocation.
- System trust means the selected SSL distribution's trust paths unless a platform
  integration says otherwise.
- The blocking API has no handshake timeout until TCP/deadline support exists.
- A `TlsStream` is not safe for concurrent operations in version one.
- Nonblocking TCP streams and post-handshake client authentication are unsupported in
  version one.
- The backend version is part of deployment. Rpath/library discovery must not select
  a different `libcrypto` than the `libssl` build expects.
- `insecure_skip_verify` is for controlled testing or an application which performs
  a separate authenticated verification mechanism; it is not a workaround for
  missing root configuration.

## 15. Decisions recommended by this study

1. Build the first implementation on the existing `TcpStream`/`TcpListener`.
2. Keep `TlsStream`/`TlsListener` separate and do not change existing TCP semantics.
3. Make `Tls.Error`, with lossless `Io` and backend-neutral `Ssl` branches, implement
   the standard `Error` trait.
4. Return the standard `Error` object from all fallible TLS APIs and implement the
   ordinary `Reader`, `Writer`, and `Closer` traits directly on `TlsStream`.
5. Preserve the complete `Tls.Error` in that object. Callers use `is Tls.Error`
   before an exhaustive domain match; do not add parallel TLS I/O traits or an
   additional I/O-error adapter layer.
6. Keep I/O traits and effect interfaces concrete rather than generic over error
   types; their concrete error type is the open `Error` object.
7. Build the PoC against the already-installed Homebrew OpenSSL 3.6.2, with a runtime
   identity guard; do not install, downgrade, or add a second SSL distribution.
8. Use two memory BIOs; do not use `SSL_set_fd` and do not implement a custom BIO in
   version one.
9. Bind exported functions directly and translate only the required public header
   macros into versioned Siko code.
10. Compile configs into immutable native contexts and keep callback roots reachable.
11. Use lazy handshakes for listener `accept`, with an explicit handshake method.
12. Treat unauthenticated TCP EOF as `UnexpectedEof`; never ignore it by default.
13. Package native TLS separately from ordinary `Std`, and select exactly one backend
    at build time.
14. For a same-semantics LibreSSL backend, use low-level `libssl`; consider `libtls`
    only as a consciously smaller alternative with less diagnostic depth.
15. Defer async TLS until the blocking pump exposes and tests its internal readiness
    transitions.

## 16. Primary external references

Only upstream project documentation/release material is used here for native-library
behavior.

### Go API reference

- [Current Go `crypto/tls` package API](https://pkg.go.dev/crypto/tls)
- [Current Go `crypto/tls` configuration source](https://go.dev/src/crypto/tls/common.go)

### Rust I/O comparison

- [Rust `Read` trait](https://doc.rust-lang.org/std/io/trait.Read.html)
- [Rust `Write` trait](https://doc.rust-lang.org/std/io/trait.Write.html)
- [Rust `io::Error` custom payload and downcasting](https://doc.rust-lang.org/std/io/struct.Error.html)
- [rustls stream implementation](https://docs.rs/rustls/latest/src/rustls/stream.rs.html)
- [flate2 gzip writer implementation](https://docs.rs/flate2/latest/src/flate2/gz/write.rs.html)

### OpenSSL

- [OpenSSL release strategy and ABI policy](https://www.openssl-library.org/policies/releasestrat/)
- [OpenSSL source releases and support dates](https://www.openssl-library.org/source/)
- [`SSL_get_error` and error-queue sequencing](https://docs.openssl.org/3.6/man3/SSL_get_error/)
- [Peer-verification flags and callback behavior](https://docs.openssl.org/3.6/man3/SSL_CTX_set_verify/)
- [Memory BIO behavior](https://docs.openssl.org/3.6/man3/BIO_s_mem/)
- [`SSL_set_bio`, `SSL_set0_rbio`, and `SSL_set0_wbio` ownership](https://docs.openssl.org/3.6/man3/SSL_set_bio/)
- [Hostname/IP verification with `SSL_set1_host`](https://docs.openssl.org/3.6/man3/SSL_set1_host/)
- [ALPN configuration and callback behavior](https://docs.openssl.org/3.6/man3/SSL_CTX_set_alpn_select_cb/)
- [`SSL_shutdown` state and return values](https://docs.openssl.org/3.6/man3/SSL_shutdown/)
- [OpenSSL error queue retrieval](https://docs.openssl.org/3.6/man3/ERR_get_error/)
- [Runtime library identity/version](https://docs.openssl.org/3.6/man3/OpenSSL_version/)

### LibreSSL and `libtls`

- [LibreSSL releases and support schedule](https://www.libressl.org/releases.html)
- [LibreSSL 4.3.2 release notes](https://ftp.openbsd.org/pub/OpenBSD/LibreSSL/libressl-4.3.2-relnotes.txt)
- [LibreSSL `SSL_get_error`](https://man.openbsd.org/SSL_get_error.3)
- [LibreSSL memory BIO behavior](https://man.openbsd.org/BIO_s_mem.3)
- [LibreSSL hostname verification](https://man.openbsd.org/SSL_set1_host.3)
- [LibreSSL ALPN](https://man.openbsd.org/SSL_CTX_set_alpn_select_cb.3)
- [LibreSSL SNI macros/callbacks](https://man.openbsd.org/SSL_CTX_set_tlsext_servername_callback.3)
- [LibreSSL protocol min/max setters](https://man.openbsd.org/SSL_CTX_set_min_proto_version.3)
- [LibreSSL cipher-list behavior, including TLS 1.3](https://man.openbsd.org/SSL_CTX_set_cipher_list.3)
- [LibreSSL compatibility version behavior](https://man.openbsd.org/OPENSSL_VERSION_NUMBER.3)
- [LibreSSL error queue](https://man.openbsd.org/ERR_get_error.3)
- [LibreSSL SSL/BIO ownership](https://man.openbsd.org/SSL_set_bio.3)
- [`libtls` client callback connections](https://man.openbsd.org/tls_connect.3)
- [`libtls` server callback accepts](https://man.openbsd.org/tls_accept_socket.3)
- [`libtls` handshake/read/write retry states](https://man.openbsd.org/tls_read.3)
- [`libtls` CA configuration, including memory input](https://man.openbsd.org/tls_config_set_ca_file.3)
- [`libtls` certificate/key configuration, including memory input](https://man.openbsd.org/tls_config_set_cert_file.3)
- [`libtls` comma-delimited ALPN configuration](https://man.openbsd.org/tls_config_set_alpn.3)
- [`libtls` client-certificate verification modes](https://man.openbsd.org/tls_load_file.3)
- [`libtls` error strings](https://man.openbsd.org/tls_error.3)
- [`libtls` connection-state inspection](https://man.openbsd.org/tls_conn_version.3)
