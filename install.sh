#!/bin/bash

# Detect Operating System
OS="$(uname -s)"
echo "Detected OS: $OS"

# Function to install Rust
install_rust() {
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
}

# Function to clone the Git repository
clone_repo() {
    echo "Cloning repository..."
    git clone https://github.com/OmarHosamCodes/wp-spotlight.git
    cd your_repo || exit
}

# Function to install dependencies
install_dependencies() {
    echo "Installing dependencies..."
    if [[ "$OS" == "Linux" ]]; then
        if command -v pacman &> /dev/null; then
            echo "Arch Linux detected."
            sudo pacman -Syu --noconfirm base-devel openssl pkg-config
        elif command -v apt &> /dev/null; then
            echo "Debian-based system detected."
            sudo apt update
            sudo apt install -y build-essential pkg-config libssl-dev
        else
            echo "Unsupported Linux distribution. Install dependencies manually."
            exit 1
        fi
    elif [[ "$OS" == "Darwin" ]]; then
        echo "macOS detected."
        brew update
        brew install openssl pkg-config
    elif [[ "$OS" =~ ^MINGW|^CYGWIN ]]; then
        echo "Windows detected. Ensure dependencies are installed manually if not present."
        # Add Windows-specific dependency installation commands if required
    else
        echo "Unsupported OS."
        exit 1
    fi
}

# Function to build the project
build_project() {
    echo "Building the project..."
    cargo build --release
    sudo mv target/release/wp-spotlight /usr/local/bin/
    echo "Project installed successfully!"
}

# Display help message
echo "Usage: ./install.fish"
wp-spotlight --help

# Main script
install_rust
install_dependencies
clone_repo
build_project
