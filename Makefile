SIKO_COMMON_SK := $(shell find siko/Common std -name '*.sk')
SIKO_COMPILER_SK := $(shell find siko/Compiler -name '*.sk') $(SIKO_COMMON_SK)
SIKO_LSP_SK := $(shell find siko/LSP -name '*.sk') $(SIKO_COMMON_SK)
HTTPD_SK := $(shell find httpd -name '*.sk')
TESTRUNNER_SK := $(shell find testrunner -name '*.sk')
SSG_SK := $(shell find ssg -name '*.sk')

SIKO_ROOT ?= $(CURDIR)
SIKO_TARGET_OS ?= macos
export SIKO_ROOT
export SIKO_TARGET_OS

BOOTSTRAP_SOURCE_PREFIX = bootstrap/source_$(SIKO_TARGET_OS)
BOOTSTRAP_SOURCE_OBJS = $(sort $(wildcard $(BOOTSTRAP_SOURCE_PREFIX).*.o))
ifeq ($(BOOTSTRAP_SOURCE_OBJS),)
BOOTSTRAP_SOURCE_OBJS = $(sort $(wildcard $(BOOTSTRAP_SOURCE_PREFIX).o))
endif

.PHONY: test

test: siko.bin runner.bin
	./siko.bin test std/Common
	./runner.bin

test-valgrind: runner.bin
	./runner.bin --valgrind

siko.bin: base.bin $(SIKO_COMPILER_SK)
	./base.bin build siko/Compiler -O -o siko.bin --trace

siko-lsp.bin: siko.bin $(SIKO_LSP_SK)
	./siko.bin build siko/LSP -O -o siko-lsp.bin --trace

.PHONY: check
check: base.bin $(SIKO_COMPILER_SK)
	./base.bin check siko/Compiler

siko2.bin: siko.bin
	./siko.bin build siko/Compiler -O -o siko2.bin --trace

siko3.bin: siko2.bin
	./siko2.bin build siko/Compiler -O -o siko3.bin --trace

base.bin: $(BOOTSTRAP_SOURCE_OBJS) link.py
	@test -n "$(BOOTSTRAP_SOURCE_OBJS)" || { echo "No bootstrap objects found for $(BOOTSTRAP_SOURCE_PREFIX)"; exit 1; }
	./link.py -o base.bin $(BOOTSTRAP_SOURCE_OBJS)

.PHONY: refresh
refresh:
	rm -f bootstrap/source_linux.o bootstrap/source_linux.*.o
	SIKO_TARGET_OS=linux ./siko.bin build siko/Compiler -O -c -o bootstrap/source_linux
	rm -f bootstrap/source_macos.o bootstrap/source_macos.*.o
	SIKO_TARGET_OS=macos ./siko.bin build siko/Compiler -O -c -o bootstrap/source_macos

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
