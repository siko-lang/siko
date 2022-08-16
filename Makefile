rust_sikoc:
	@build/build_rust_sikoc.sh

stage0: rust_sikoc $(shell find sikoc/src -type f)
	@build/build_stage0.sh

stage1: stage0 $(shell find sikoc/src -type f)
	@build/build_stage1.sh

stage2: stage1 $(shell find sikoc/src -type f)
	@build/build_stage2.sh

stage0_test: stage0
	@./tests stage0

stage1_test: stage1
	@./tests stage1

minimal:
	@sikoc/rr

clean:
	@rm -f rust_sikoc
	@rm -f stage0
	@rm -f stage1
	@rm -f stage2
	@rm -rf build/stage0
	@rm -rf build/stage1
	@rm -rf build/stage2