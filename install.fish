#!/usr/bin/env fish

# Build the release version
cargo build --release

# Create binary directory if it doesn't exist
mkdir -p ~/.local/bin

# Copy the binary
cp target/release/wp-spotlight ~/.local/bin/

# Make it executable
chmod +x ~/.local/bin/wp-spotlight

# Add to PATH if not already there
if not contains "$HOME/.local/bin" $fish_user_paths
    set -Ua fish_user_paths $HOME/.local/bin
    echo "Added ~/.local/bin to PATH. Please restart your terminal or run:"
    echo "exec fish"
end

echo "wp-spotlight has been installed successfully!"
