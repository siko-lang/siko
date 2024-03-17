test: Siko/target/release/siko
	@./siko test.sk

Siko/target/release/siko: $(shell find Siko/src/ -type f)
	@cd Siko && cargo build --release

teststd: Siko/target/release/siko
	@./siko test.sk std/*
