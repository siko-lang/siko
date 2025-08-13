compiler/target/release/siko: $(shell find compiler/src/ -type f)
	@cd compiler && cargo build --release

simple: compiler/target/release/siko
	@./siko test.sk

fmt:
	@cd compiler && cargo fmt

clean:
	@cd compiler && cargo clean

teststd: compiler/target/release/siko
	@./siko run test.sk std/*

c: compiler/target/release/siko
	@./siko run test.sk

test: compiler/target/release/siko
	@./run_test.py

testworkflow: compiler/target/release/siko
	sudo apt install -y valgrind
	@./run_test.py --workflow