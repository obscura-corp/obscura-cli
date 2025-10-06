<div align="center">

# <span style="color: #ff006e;">obscuraCLI</span>

<img src="https://img.shields.io/github/v/release/obscura-os/obscura-cli?color=8338ec&style=for-the-badge"> <img src="https://img.shields.io/github/stars/obscura-os/obscura-cli?color=8338ec&style=for-the-badge">

**Local API key vault for developers. No cloud. No telemetry. Local dev only.**

</div>

A single-binary, cross-platform local API-key vault CLI secured only by a passphrase. Obscura provides secure storage for API keys and secrets with strong encryption, supporting both global and project-specific vaults.

## Features

- **Passphrase-secured encryption** using Argon2id KDF and XChaCha20-Poly1305 AEAD
- **Local-only storage** - no cloud, no telemetry, no networking
- **Cross-platform** - works on macOS, Linux, and Windows
- **Dual vault system** - global vault and project-specific vaults
- **Secure session caching** - encrypted key caching to avoid repeated passphrase prompts
- **Secret rotation** - update secrets while preserving creation timestamps
- **Export capabilities** - export secrets as dotenv format
- **Run code without .env** - obscura can directly inject secrets into your run command so you never need to use a .env
- **Vault cleanup** - safely delete project or global vaults when no longer needed
- **Zero plaintext on disk** - all data encrypted with strict file permissions

## Installation

### Curl from website (macOS and Linux only)

```bash
curl -fsSL https://www.obscura.team/install.sh | sh
```

### From Repository (requires rust)

```bash
git clone https://github.com/obscura-corp/obscura-cli
cd obscura-cli
cargo build --release
```

## Quick Start

### Global Vault (Auto-created)

```bash
# Add a secret (global vault is auto-created on first use)
obscura add openai

# Retrieve a secret
obscura get openai

# List all secrets
obscura list
```

### Project Vault

```bash
# Navigate to your project directory
cd myapp

# Initialize a project vault (default behavior)
obscura init

# Add secrets to the project vault
obscura add stripe
obscura add stripe --from-global  # Copy from global vault

# List project secrets
obscura list

# Export as dotenv
obscura export --dotenv --output .env.local
```

## Commands

### `obscura init [--global]`

Initialize a vault (project by default, global with flag).

**Options:**
- `--global` - Initialize the global vault

**Examples:**
```bash
obscura init                    # Initialize project vault for current directory
obscura init --global           # Initialize global vault
```

### `obscura add <alias> [OPTIONS]`

Add a secret to the vault.

**Arguments:**
- `<alias>` - Alias name for the secret

**Options:**
- `--global` - Operate on the global vault
- `--project` - Operate on the project vault
- `--from-global` - Copy the alias from the global vault into the project vault

**Examples:**
```bash
obscura add openai                    # Add to project vault (if exists) or global
obscura add stripe --global           # Add to global vault
obscura add stripe --from-global      # Copy from global to project vault
```

### `obscura get <alias> [OPTIONS]`

Retrieve a secret from the vault.

**Arguments:**
- `<alias>` - Alias name to retrieve

**Options:**
- `--global` - Read from the global vault
- `--project` - Read from the project vault

**Examples:**
```bash
obscura get openai                    # Get from project vault (if exists) or global
obscura get stripe --global           # Get from global vault
```

### `obscura list [OPTIONS]`

List secrets in the vault.

**Options:**
- `--global` - List entries from the global vault
- `--project` - List entries from the project vault
- `--json` - Render output as JSON

**Examples:**
```bash
obscura list                          # List from project vault (if exists) or global
obscura list --json                   # List as JSON format
obscura list --global                 # List from global vault
```

### `obscura remove <alias> [OPTIONS]`

Remove a secret from the vault.

**Arguments:**
- `<alias>` - Alias name to remove

**Options:**
- `--global` - Remove from the global vault
- `--project` - Remove from the project vault
- `--yes` - Skip the confirmation prompt

**Examples:**
```bash
obscura remove openai --yes           # Remove without confirmation
obscura remove stripe --global        # Remove from global vault
```

### `obscura delete [OPTIONS]`

Delete an entire vault and all stored secrets.

**Options:**
- `--global` - Delete the global vault
- `--project` - Delete the project vault for the current directory
- `--yes` - Skip the confirmation prompt

**Examples:**
```bash
obscura delete --project --yes        # Delete project vault without prompt
obscura delete --global               # Delete global vault (with confirmation)
```

### `obscura rotate <alias> [OPTIONS]`

Rotate (update) a secret in the vault.

**Arguments:**
- `<alias>` - Alias name to rotate

**Options:**
- `--global` - Rotate in the global vault
- `--project` - Rotate in the project vault

**Examples:**
```bash
obscura rotate openai                 # Rotate in project vault (if exists) or global
obscura rotate stripe --global        # Rotate in global vault
```

### `obscura export --dotenv [OPTIONS]`

Export secrets as dotenv content.

**Options:**
- `--global` - Export from the global vault
- `--project` - Export from the project vault
- `--output <path>` - Write output to this file path
- `--overwrite` - Allow overwriting the output file

**Examples:**
```bash
obscura export --dotenv                        # Print dotenv to stdout
obscura export --dotenv --output .env.local    # Write to file
obscura export --dotenv --global --output .env # Export global vault
```

### `obscura run <command> [OPTIONS]`

Run a command with secrets injected as environment variables.

**Arguments:**
- `<command>` - Command and arguments to execute (after --)

**Options:**
- `--global` - Use the global vault
- `--project` - Use the project vault

**Examples:**
```bash
obscura run -- npm start                    # Run npm start with project secrets
obscura run -- python app.py               # Run Python app with secrets
obscura run --global -- node server.js     # Run with global vault secrets
```

### `obscura unlock [OPTIONS]`

Cache vault keys for a limited time to avoid repeated passphrase prompts.

**Options:**
- `--timeout <minutes>` - Cache timeout in minutes (default: 60)
- `--global` - Target the global vault
- `--project` - Target the project vault for the current directory

**Examples:**
```bash
obscura unlock --timeout 30           # Cache for 30 minutes
obscura unlock --global               # Cache global vault
```

### `obscura lock [OPTIONS]`

Clear cached vault keys.

**Options:**
- `--global` - Target the global vault
- `--project` - Target the project vault for the current directory

**Examples:**
```bash
obscura lock                          # Clear all cached keys
obscura lock --global                 # Clear global vault cache
```

## Vault Resolution

Obscura uses a smart vault resolution system:

1. **Project vault** - If you're in a directory with a project vault, it's used by default
2. **Global vault** - Falls back to the global vault if no project vault exists
3. **Force flags** - Use `--global` or `--project` to override the default behavior

## Security

### Encryption

- **KDF**: Argon2id with configurable parameters (default: 64MB memory, 1 iteration)
- **AEAD**: XChaCha20-Poly1305 for authenticated encryption
- **Key derivation**: 32-byte random DEK wrapped by KEK derived from passphrase
- **Nonces**: 24-byte random nonces for each encryption operation

### File Security

- **Zero plaintext on disk** - all data is encrypted, including session cache
- **Strict permissions** - 0600 (user-only) on Unix systems
- **Atomic writes** - temporary files prevent corruption during writes
- **Encrypted session storage** - cached keys are encrypted with system-specific derivation
- **File locking** - prevents race conditions during concurrent access
- **No recovery** - lost passphrase means lost data (by design)

### Session Caching

- **Encrypted key caching** - DEKs encrypted with system-specific key derivation
- **Secure storage** - session data encrypted and stored with proper permissions
- **File locking** - prevents concurrent access corruption
- **Automatic cleanup** - expired sessions are automatically removed

## Configuration

### Environment Variables

- `OBSCURA_PASSPHRASE` - Set passphrase for non-interactive use (CI/testing only)
- `OBSCURA_KDF_MEM_KIB` - Override KDF memory usage (64-524 MB)
- `OBSCURA_KDF_TIME` - Override KDF time parameter (1-6 iterations)

### File Locations

**macOS:**
- Global vault: `~/Library/Application Support/Obscura/vault.enc`
- Project vaults: `~/Library/Application Support/Obscura/projects/<hash>/vault.enc`

**Linux:**
- Global vault: `$XDG_CONFIG_HOME/obscura/vault.enc` (fallback: `~/.config/obscura/vault.enc`)
- Project vaults: `$XDG_CONFIG_HOME/obscura/projects/<hash>/vault.enc`

**Windows:**
- Global vault: `%APPDATA%\Obscura\vault.enc`
- Project vaults: `%APPDATA%\Obscura\projects\<hash>\vault.enc`

## Exit Codes

- `0` - Success
- `1` - General error
- `2` - Alias not found

## Examples

### Development Workflow

```bash
# Set up project vault
cd myapp
obscura init

# Add API keys
obscura add openai_api_key
obscura add stripe_secret_key

# Cache keys for development session
obscura unlock --timeout 120

# Use in development
export OPENAI_API_KEY=$(obscura get openai_api_key)
export STRIPE_SECRET_KEY=$(obscura get stripe_secret_key)

# Export for deployment
obscura export --dotenv --output .env.production

# Clean up
obscura lock
```

### Python example

```bash
# Set up project vault
cd myPythonApp
obscura init

# Add API keys
obscura add openai_api_key

# Cache keys for development session
obscura unlock --timeout 120

# Use in development
import os
openai_key = os.getenv('openai_api_key')

# Export for deployment
obscura run -- python3 app.py

# Clean up after done
obscura lock
```

### Global vs Project Secrets

```bash
# Add to global vault
obscura add github_token --global

# Copy to project
cd myapp
obscura add github_token --from-global

# Project-specific secret
obscura add myapp_database_url

# List project secrets
obscura list
```

## Security Notice

⚠️ **This tool is for local development only.** It is not designed for production use or team collaboration. Always use proper secret management solutions for production environments.

⚠️ **No recovery mechanism.** If you lose your passphrase, your data is permanently lost. Make sure to back up your passphrase securely.

## Contributing

Contributions are welcome! Please read our contributing guidelines and submit pull requests to our [GitHub repository](https://github.com/obscura-corp/obscura-cli).

## Support

For issues and questions, please use the [GitHub Issues](https://github.com/obscura-corp/obscura-cli/issues) page.
