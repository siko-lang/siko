SIKO_SK := $(shell find siko std -name '*.sk')
HTTPD_SK := $(shell find httpd -name '*.sk')
TESTRUNNER_SK := $(shell find testrunner -name '*.sk')
SSG_SK := $(shell find ssg -name '*.sk')

SIKO_ROOT ?= $(CURDIR)
SIKO_TARGET_OS ?= macos
export SIKO_ROOT
export SIKO_TARGET_OS

# The LLVM backend emits fatter stack frames than the C backend, so the
# compiler's recursion needs more than the default 8MB main stack. macOS
# handles this at link time (see link.py); on Linux only ulimit can raise it.
# Every recipe shell sources stack_limit.sh via BASH_ENV.
SHELL := /bin/bash
export BASH_ENV := $(CURDIR)/stack_limit.sh

BOOTSTRAP_SOURCE_C = bootstrap/source_$(SIKO_TARGET_OS).c
BOOTSTRAP_SOURCE_LL = bootstrap/source_$(SIKO_TARGET_OS).ll

.PHONY: test

test: siko.bin runner.bin
	./siko.bin test std/Common
	./runner.bin
	./runner.bin --llvm

test-valgrind: runner.bin
	./runner.bin --valgrind

siko.bin: base_ll.bin $(SIKO_SK)
	./base_ll.bin build siko -O -o siko.bin

.PHONY: check
check: base_ll.bin $(SIKO_SK)
	./base_ll.bin check siko

siko2.bin: siko.bin
	./siko.bin build siko -O -o siko2.bin

siko3.bin: siko2.bin
	./siko2.bin build siko -O -o siko3.bin

base_c.bin: $(BOOTSTRAP_SOURCE_C)
	cat $(BOOTSTRAP_SOURCE_C) | ./link.py -O -o base_c.bin

base_ll.bin: $(BOOTSTRAP_SOURCE_LL)
	cat $(BOOTSTRAP_SOURCE_LL) | ./link.py --llvm -O -o base_ll.bin

.PHONY: refresh
refresh:
	SIKO_TARGET_OS=linux ./siko.bin build siko --pass c > bootstrap/source_linux.c
	SIKO_TARGET_OS=linux ./siko.bin build siko --llvm --pass llvm > bootstrap/source_linux.ll
	SIKO_TARGET_OS=macos ./siko.bin build siko --pass c > bootstrap/source_macos.c
	SIKO_TARGET_OS=macos ./siko.bin build siko --llvm --pass llvm > bootstrap/source_macos.ll

ssg.bin: siko.bin $(SSG_SK)
	./siko.bin build ssg -o ssg.bin

.PHONY: site web
site: ssg.bin
	./ssg.bin build docs

web: site
	python3 docs/output/server.py

runner.bin: siko.bin ${TESTRUNNER_SK}
	./siko.bin build testrunner -o runner.bin

httpd: siko.bin $(HTTPD_SK)
	./siko.bin build httpd -o httpd.bin
