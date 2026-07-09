SIKO_SK := $(shell find siko std -name '*.sk')
HTTPD_SK := $(shell find httpd -name '*.sk')
TESTRUNNER_SK := $(shell find testrunner -name '*.sk')
SSG_SK := $(shell find ssg -name '*.sk')

SIKO_ROOT ?= $(CURDIR)
SIKO_TARGET_OS ?= macos
export SIKO_ROOT
export SIKO_TARGET_OS

BOOTSTRAP_SOURCE = bootstrap/source_$(SIKO_TARGET_OS).c

.PHONY: test

test: siko.bin runner.bin
	./siko.bin test std/Common
	./runner.bin
	./runner.bin --llvm

test-valgrind: runner.bin
	./runner.bin --valgrind

siko.bin: base.bin $(SIKO_SK)
	./base.bin build siko -O -o siko.bin

.PHONY: check
check: base.bin $(SIKO_SK)
	./base.bin check siko

siko2.bin: siko.bin
	./siko.bin build siko -O -o siko2.bin

siko3.bin: siko2.bin
	./siko2.bin build siko -O -o siko3.bin

base.bin: $(BOOTSTRAP_SOURCE)
	cat $(BOOTSTRAP_SOURCE) | ./link.sh -O -o base.bin

refresh:
	SIKO_TARGET_OS=linux ./siko.bin build siko --pass c > bootstrap/source_linux.c
	SIKO_TARGET_OS=macos ./siko.bin build siko --pass c > bootstrap/source_macos.c

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
