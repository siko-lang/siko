TESTRUNNER = testrunner/target/debug/testrunner

SIKO_SK := $(shell find siko std -name '*.sk')

SIKO_TARGET_OS ?= darwin
BOOTSTRAP_SOURCE = bootstrap/source_$(SIKO_TARGET_OS).c

.PHONY: build test all

all: build

build:
	cd testrunner; cargo build --workspace

test: build
	./$(TESTRUNNER)

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
	./siko.bin build siko --pass c > $(BOOTSTRAP_SOURCE)

SSG_SK := $(shell find ssg -name '*.sk')

ssg.bin: siko.bin $(SSG_SK)
	./siko.bin build ssg -o ssg.bin

.PHONY: site web
site: ssg.bin
	./ssg.bin build docs

web: site
	python3 docs/output/server.py