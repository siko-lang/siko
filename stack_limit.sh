# Sourced by every Makefile recipe shell (via BASH_ENV, see Makefile).
# Raise the stack soft limit as far as the platform allows: the LLVM-built
# compiler recurses deeper than the default 8MB main stack. Linux allows
# 256MB (hard limit is unlimited); macOS caps the hard limit at ~64MB but
# also sets the real limit at link time (see link.py), so failing is fine.
ulimit -s 262144 2>/dev/null || ulimit -s hard 2>/dev/null || true
