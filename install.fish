#!/usr/bin/env fish

# Detect Operating System
set -l OS (uname -s)
echo "Detected OS: $OS"

# Function to install Rust
function install_rust
    echo "Installing Rust..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
    source $HOME/.cargo/env
end

# Function to clone the Git repository
function clone_repo
    echo "Cloning repository..."
    git clone https://github.com/OmarHosamCodes/wp-spotlight.git
    cd wp-spotlight; or exit
end

# Function to install dependencies
function install_dependencies
    echo "Installing dependencies..."
    if test "$OS" = "Linux"
        if type -q pacman
            echo "Arch Linux detected."
            sudo pacman -Syu --noconfirm base-devel openssl pkg-config
        else if type -q apt
            echo "Debian-based system detected."
            sudo apt update
            sudo apt install -y build-essential pkg-config libssl-dev
        else
            echo "Unsupported Linux distribution. Install dependencies manually."
            exit 1
        end
    else if test "$OS" = "Darwin"
        echo "macOS detected."
        brew update
        brew install openssl pkg-config
    else if string match -rq '^MINGW|^CYGWIN' -- "$OS"
        echo "Windows detected. Ensure dependencies are installed manually if not present."
        # Add Windows-specific dependency installation commands if required
    else
        echo "Unsupported OS."
        exit 1
    end
end

# Function to build the project
function build_project
    echo "Building the project..."
    cargo build --release
    sudo mv target/release/wp-spotlight /usr/local/bin/
    echo "Project installed successfully!"
end

# Display help message
echo "Usage: ./install.fish"
wp-spotlight --help

# Main script
install_rust
install_dependencies
clone_repo
build_project
