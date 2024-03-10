test: Siko/target/release/
	cd Siko && cargo run --release && cd .. && ./siko test.sk

test2: Siko/target/release/
	cd Siko && cargo run --release && cd .. && ./siko std/*