test: Siko/target/release/siko
	@./siko test.sk

Siko/target/release/siko: $(shell find Siko/src/ -type f)
	@cd Siko && cargo build --release

teststd: Siko/target/release/siko
	@./siko test.sk std/*

llvm: Siko/target/release/siko
	@./siko test.sk
	@clang -Wno-override-module llvm.ll -o llvm_main.bin