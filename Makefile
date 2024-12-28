test: Siko/target/release/siko
	@./siko test.sk

Siko/target/release/siko: $(shell find Siko/src/ -type f)
	@cd Siko && cargo build --release

clean:
	@cd Siko && cargo clean

teststd: Siko/target/release/siko
	@./siko test.sk std/*

siko_runtime/siko_runtime.o: $(shell find siko_runtime -type f -name *.c)
	siko_runtime/build.sh

llvm: Siko/target/release/siko siko_runtime/siko_runtime.o
	@./siko test.sk
	@opt -O2 -S llvm.ll -o optimized.ll
	@llvm-as optimized.ll -o main.bc
	@llc -relocation-model=pic main.bc -filetype=obj -o main.o
	@clang main.o siko_runtime/siko_runtime.o -o main.bin

c: Siko/target/release/siko siko_runtime/siko_runtime.o
	@./siko test.sk
	@clang -c siko_main.c -o siko_main.o -I siko_runtime
	@clang siko_main.o siko_runtime/siko_runtime.o -o main.bin
	@./main.bin
