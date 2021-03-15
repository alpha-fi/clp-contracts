test:
# to test specific test run: cargo test <test name>
	@cargo test

test-debug:
# "--" allows to pass extra arguments
# "--nocapture" disables stdout capturing (testes print all println)
	@RUST_BACKTRACE=1 cargo test  -- --nocapture

test-unit:
	@cargo test --lib
# run specific tests: cargo test --lib <testname>  -- --nocapture


build:
# more about flags: https://github.com/near-examples/simulation-testing#gotchas
# env setting instruments cargo to optimize the the build for size (link-args=-s)
	@env 'RUSTFLAGS=-C link-arg=-s' cargo build --lib --target wasm32-unknown-unknown --release
	@cd ..; cp target/wasm32-unknown-unknown/release/*.wasm ./res/
