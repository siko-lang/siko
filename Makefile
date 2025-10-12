compiler/target/release/siko: $(shell find compiler/src/ -type f)
	@cargo build --release
	@cp target/release/siko .

simple: compiler/target/release/siko
	@./siko test.sk

fmt:
	@cargo fmt

clean:
	@cargo clean
	@rm -f siko

teststd: compiler/target/release/siko
	@./siko run test.sk std/*

c: compiler/target/release/siko
	@./siko run test.sk

test: compiler/target/release/siko
	@./run_test.py

stdtest: compiler/target/release/siko
	@./siko test ./std

testworkflow: compiler/target/release/siko
	sudo apt update
	sudo apt install -y valgrind
	@./run_test.py --workflow

testrunner: compiler/target/release/siko
	@./siko build testrunner -o testrunner.bin