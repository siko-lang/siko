stage0: bootstrap/source.rs
	@rustc bootstrap/source.rs -O -o stage0

stage1: stage0 $(shell find src -type f)
	@./stage0 ./src ./std -v -o ./stage1

stage2: stage1 $(shell find src -type f)
	@./stage1 ./src ./std -v -o ./stage2

siko: stage0 $(shell find incremental -type f)
	@./stage0 ./incremental/src ./std -o ./siko

test: stage1
	@./tests stage1

clean:
	@rm -f stage0
	@rm -f stage1
	@rm -f stage2
	@rm -f siko
	@rm -rf build/stage0
	@rm -rf build/stage1
	@rm -rf build/stage2
	@rm -rf build/siko