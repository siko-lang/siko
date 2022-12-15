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

clean:
	@rm -f stage0
	@rm -f stage1
	@rm -f stage2
	@rm -f siko
	@rm -rf testrunner