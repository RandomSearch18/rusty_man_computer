# A simple shell script for building binaries, in preparation to be uploaded to GitHub releases.
# Tested on Arch Linux
# Requirements: `cross` (cross-rs)
# May require targets to be added using `rustup target add <target-triple>`

targets=(
    x86_64-unknown-linux-gnu
    x86_64-pc-windows-gnu
    # x86_64-apple-darwin
    aarch64-unknown-linux-gnu
)

version=$(cargo run --bin rusty_man_computer -- --version | cut -d ' ' -f 2)

# Clear the `target/dist` directory
mkdir -p target/dist
rm target/dist/*

for target in "${targets[@]}"; do
    echo "Building for $target"
    cross build --release --target "$target"
    # Copy both binaries to `target/dist`
    cp "target/$target/release/rusty_man_computer" "target/dist/rusty-man-computer-$version-$target" || \
        cp "target/$target/release/rusty_man_computer.exe" "target/dist/rusty-man-computer-$version-$target.exe"
    cp "target/$target/release/bin_creator" "target/dist/bin-creator-$version-$target" || \
        cp "target/$target/release/bin_creator.exe" "target/dist/bin-creator-$version-$target.exe"
    cp "target/$target/release/rmc_assemble" "target/dist/rmc-assemble-$version-$target" || \
        cp "target/$target/release/rmc_assemble.exe" "target/dist/rmc-assemble-$version-$target.exe"
done
