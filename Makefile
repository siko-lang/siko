compiler/target/release/siko: $(shell find compiler/src/ -type f)
	@cargo build --release
	@cp target/release/siko .

.PHONY: fmt
fmt:
	@cargo fmt

.PHONY: clean
clean:
	@cargo clean
	@rm -f siko

.PHONY: test
test: compiler/target/release/siko
	@./legacy_run_test.py

.PHONY: testnew
testnew: runner.bin
	@./runner.bin

.PHONY: stdtest
stdtest: compiler/target/release/siko
	@./siko test ./std

testworkflow: compiler/target/release/siko
	sudo apt update
	sudo apt install -y valgrind
	@./legacy_run_test.py --workflow

runner.bin: compiler/target/release/siko
	@./siko build testrunner -o runner.bin

self.bin: compiler/target/release/siko
	@./siko build compiler2 -o self.bin