export PATH := $(PWD)/bin:$(PATH)

bin/stage0: bootstrap/source.rs
	@mkdir -p bin
	@rustc bootstrap/source.rs -O -o bin/stage0

bin/stage1: bin/stage0 $(shell find src -type f)
	@stage0 build ./src ./std -v -o bin/stage1

bin/stage2: bin/stage1 $(shell find src -type f)
	@stage1 build ./src ./std -v -o bin/stage2

bin/siko: bin/stage0 $(shell find incremental -type f)
	@stage0 build ./incremental/src ./std -v -o bin/siko

bin/testrunner: bin/stage0 $(shell find test_runner -type f)
	@stage0 build ./test_runner ./std -o bin/testrunner

bin/multi_parser: bin/stage1 $(shell find multistage/Common multistage/Parser -type f)
	@echo "M: Parser"
	@stage1 build multistage/Parser ./std -o bin/multi_parser

bin/multi_nameresolver: bin/stage1 $(shell find multistage/Common multistage/NameResolver -type f)
	@echo "M: NameResolver"
	@stage1 build multistage/NameResolver ./std -o bin/multi_nameresolver

bin/multi_typechecker: bin/stage1 $(shell find multistage/Common multistage/Typechecker -type f)
	@echo "M: Typechecker"
	@stage1 build multistage/Typechecker ./std -o bin/multi_typechecker

bin/multi_hirbackend: bin/stage1 $(shell find multistage/Common multistage/HIRBackend -type f)
	@echo "M: HIRBackend"
	@stage1 build multistage/HIRBackend ./std -o bin/multi_hirbackend

bin/multi_mirlowering: bin/stage1 $(shell find multistage/Common multistage/MIRLowering -type f)
	@echo "M: MIRLowering"
	@stage1 build multistage/MIRLowering ./std -o bin/multi_mirlowering

bin/multi_mirbackend: bin/stage1 $(shell find multistage/Common multistage/MIRBackend -type f)
	@echo "M: MIRBackend"
	@stage1 build multistage/MIRBackend ./std -o bin/multi_mirbackend

bin/multi_transpiler: bin/stage1 $(shell find multistage/Common multistage/Transpiler -type f)
	@echo "M: Transpiler"
	@stage1 build multistage/Transpiler ./std -o bin/multi_transpiler

bin/merged: bin/stage1 $(shell find multistage/Common multistage/Merged -type f)
	@stage1 build multistage/Merged ./std -o bin/merged

.PHONY: stage0
stage0: bin/stage0

.PHONY: stage1
stage1: bin/stage1

.PHONY: stage2
stage2: bin/stage2

.PHONY: testrunner
testrunner: bin/testrunner

.PHONY: multistage
multistage: bin/multi_parser bin/multi_nameresolver bin/multi_typechecker bin/multi_hirbackend bin/multi_mirlowering bin/multi_mirbackend bin/multi_transpiler

.PHONY: multistage_clean
multistage_clean:
	@rm -rf bin/multi_*

.PHONY: run_multistage
run_multistage: multistage
	@rm -rf cache
	multi_parser build multistage_test
	multi_nameresolver
	multi_typechecker
	multi_hirbackend
	multi_mirlowering
	multi_mirbackend
	multi_transpiler

.PHONY: clean
clean:
	@rm -rf bin

.PHONY: test
test: bin/stage1 bin/testrunner
	@testrunner stage1
