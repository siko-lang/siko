test: Siko/target/release/
	cd Siko && cargo build --release && cd .. && ./siko test.sk

teststd: Siko/target/release/
	cd Siko && cargo build --release && cd .. && ./siko test.sk std/*

test2: Siko/target/release/
	cd Siko && cargo run --release && cd .. && ./run_test.py