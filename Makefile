stage0: bootstrap/source.rs
	@rustc bootstrap/source.rs -O -o stage0

stage1: stage0 $(shell find src -type f)
	@./stage0 build ./src ./std -v -o ./stage1

stage2: stage1 $(shell find src -type f)
	@./stage1 build ./src ./std -v -o ./stage2

siko: stage0 $(shell find incremental -type f)
	@./stage0 build ./incremental/src ./std -v -o ./siko

test: stage1 testrunner
	@./testrunner stage1

testrunner: stage0 $(shell find test_runner -type f)
	@./stage0 build ./test_runner ./std -o ./testrunner

altfmt: stage0 $(shell find experimental/alternative_syntax -type f)
	@./stage0 build experimental/alternative_syntax ./std -v -o alt

parser2: stage1 $(shell find multistage/Common multistage/Parser -type f)
	@echo "M: Parser"
	@./stage1 build multistage/Parser ./std -o parser2

nameresolver2: stage1 $(shell find multistage/Common multistage/NameResolver -type f)
	@echo "M: NameResolver"
	@./stage1 build multistage/NameResolver ./std -o nameresolver2

typechecker2: stage1 $(shell find multistage/Common multistage/Typechecker -type f)
	@echo "M: Typechecker"
	@./stage1 build multistage/Typechecker ./std -o typechecker2

hirbackend2: stage1 $(shell find multistage/Common multistage/HIRBackend -type f)
	@echo "M: HIRBackend"
	@./stage1 build multistage/HIRBackend ./std -o hirbackend2

mirlowering2: stage1 $(shell find multistage/Common multistage/MIRLowering -type f)
	@echo "M: MIRLowering"
	@./stage1 build multistage/MIRLowering ./std -o mirlowering2

multistage: parser2 nameresolver2 typechecker2 hirbackend2 mirlowering2

run_multistage: multistage
	@rm -rf cache
	./parser2 build multistage_test
	./nameresolver2
	./typechecker2
	./hirbackend2
	./mirlowering2

clean:
	@rm -f stage0
	@rm -f stage1
	@rm -f stage2
	@rm -f siko
	@rm -rf testrunner