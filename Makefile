compiler/target/release/siko: $(shell find compiler/src/ -type f)
	@cd compiler && cargo build --release

simple: compiler/target/release/siko
	@./siko test.sk

fmt:
	@cd compiler && cargo fmt

clean:
	@cd compiler && cargo clean

teststd: compiler/target/release/siko
	@./siko test.sk std/*

llvm: compiler/target/release/siko
	@./siko test.sk
	@opt -O2 -S llvm.ll -o optimized.ll
	@llvm-as optimized.ll -o main.bc
	@llc -relocation-model=pic main.bc -filetype=obj -o main.o
	@clang main.o siko_runtime/siko_runtime.o -o main.bin

c: compiler/target/release/siko
	@./siko test.sk
	@clang siko_main.c -o main.bin
	@./main.bin

self.bin: self compiler/target/release/siko std
	@./siko self ./std -o self
	@clang self.c -o self.bin
	@./self.bin

test: compiler/target/release/siko
	@./run_test.py

testworkflow: compiler/target/release/siko
	sudo apt install -y valgrind
	@./run_test.py --workflow