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
test: runner.bin
	@./runner.bin

.PHONY: stdtest
stdtest: compiler/target/release/siko
	@./siko test ./std

runner.bin: compiler/target/release/siko
	@./siko build testrunner -o runner.bin

self.bin: compiler/target/release/siko
	@# Disable inliner because the current backend is slow and inlining makes compilation too slow.
	@./siko build compiler2 --disable-inliner -o self.bin