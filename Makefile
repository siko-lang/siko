simple: Siko/target/release/siko
	@./siko test.sk

Siko/target/release/siko: $(shell find Siko/src/ -type f)
	@cd Siko && cargo build --release

clean:
	@cd Siko && cargo clean

teststd: Siko/target/release/siko
	@./siko test.sk std/*

llvm: Siko/target/release/siko
	@./siko test.sk
	@opt -O2 -S llvm.ll -o optimized.ll
	@llvm-as optimized.ll -o main.bc
	@llc -relocation-model=pic main.bc -filetype=obj -o main.o
	@clang main.o siko_runtime/siko_runtime.o -o main.bin

c: Siko/target/release/siko
	@./siko test.sk
	@clang siko_main.c -o main.bin
	@./main.bin

self.bin: self Siko/target/release/siko std
	@./siko self ./std -o self
	@clang self.c -o self.bin
	@./self.bin

test: Siko/target/release/siko
	@./run_test.py
