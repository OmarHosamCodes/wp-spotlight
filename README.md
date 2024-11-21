
# Project Installation Script

This repository provides a script to install Rust, fetch this repository, and build the project on Linux (Debian-based, Arch), macOS, or Windows.

## Supported Platforms

- **Linux**:
  - Debian-based (e.g., Ubuntu)
  - Arch Linux
- **macOS**
- **Windows** (using Git Bash or equivalent)

## Prerequisites

- **Linux**: `curl`, `git`, `sudo`, and a package manager (e.g., `apt` or `pacman`).
- **macOS**: `curl`, `git`, and Homebrew.
- **Windows**: Git Bash or similar environment.

## How to Run

1. Clone this repository:
   ```bash
   git clone https://github.com/OmarHosamCodes/wp-spotlight.git
   cd wp-spotlight
   ```

2. Make the script executable:
   ```bash
   chmod +x install_project.sh
   ```

3. Run the script:
   ```bash
   ./install_project.sh
   ```

4. The executable will be installed to `/usr/local/bin` for system-wide access.

## Notes for Arch Linux

The script uses `pacman` to install:
- `base-devel`: For compilation tools.
- `openssl` and `pkg-config`: For managing OpenSSL.

## Troubleshooting

- Ensure you have the necessary permissions to install software and move files to `/usr/local/bin`.
- If Rust or dependencies fail to install, refer to their official installation guides.
- For unsupported Linux distributions, install dependencies manually:
  - `gcc`, `make`, `pkg-config`, and `openssl`.

---
