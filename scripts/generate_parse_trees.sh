pushd tools/generate_parse_trees > /dev/null || exit

cargo run > "../../src/parsing/parse_tree.rs"

popd > /dev/null

cargo fmt
