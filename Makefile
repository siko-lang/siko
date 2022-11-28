stage0: bootstrap/source.rs
	@rustc bootstrap/source.rs -O -o stage0

stage1: stage0 $(shell find src -type f)
	@build/build_stage1.sh

stage2: stage1 $(shell find src -type f)
	@build/build_stage2.sh

sikofmt: stage0 $(shell find fmt -type f)
	@build/build_fmt.sh

test: stage1
	@./tests stage1

clean:
	@rm -f stage0
	@rm -f stage1
	@rm -f stage2
	@rm -f sikofmt
	@rm -rf build/stage0
	@rm -rf build/stage1
	@rm -rf build/stage2
	@rm -rf build/sikofmt