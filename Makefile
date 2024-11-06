test: Siko/target/release/siko
	@./siko test.sk

Siko/target/release/siko: $(shell find Siko/src/ -type f)
	@cd Siko && cargo build --release

teststd: Siko/target/release/siko
	@./siko test.sk std/*

llvm: Siko/target/release/siko
	@./siko test.sk
	@opt -O2 -S llvm.ll -o optimized.ll
	@clang -Wno-override-module llvm.ll -o llvm_main.bin
