test: Siko/target/release/
	cd Siko && cargo build --release && cd .. && ./siko test.sk

testdbg: Siko/target/release/
	export RUST_BACKTRACE=1 && cd Siko && cargo run -- ../test.sk

test2: Siko/target/release/
	cd Siko && cargo run --release && cd .. && ./run_test.py