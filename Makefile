compiler/target/release/siko: $(shell find compiler/src/ -type f)
	@cargo build --release
	@cp target/release/siko .

simple: compiler/target/release/siko
	@./siko test.sk

.PHONY: fmt
fmt:
	@cargo fmt

.PHONY: clean
clean:
	@cargo clean
	@rm -f siko

test: compiler/target/release/siko
	@./run_test.py

.PHONY: stdtest
stdtest: compiler/target/release/siko
	@./siko test ./std

testworkflow: compiler/target/release/siko
	sudo apt update
	sudo apt install -y valgrind
	@./run_test.py --workflow

.PHONY: testrunner
testrunner: compiler/target/release/siko
	@./siko build testrunner -o testrunner.bin