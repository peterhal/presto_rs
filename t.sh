./scripts/generate_parse_trees.sh && RUST_BACKTRACE=full cargo run private/test.sql && cargo fmt && cargo run --release private/test.sql && pushd tools/generate_parse_trees && cargo fmt && popd
