# Candado

**Candado** is a modern, command-line password manager and secrets generator. The name "Candado," meaning "lock" in Spanish, reflects the core purpose of this tool: securely managing and generating passwords, tokens, and keys.

---

## Features
- **Password Management:**
  - Initialize and manage a secure vault for your passwords.
  - Add, update, list, find, inspect, and remove password entries.
  - Import and export entries in JSON format.
- **Secrets Generation:**
  - Generate strong passwords of customizable lengths.
  - Generate secure tokens and keys.
  - Generate memorable passphrases with options for custom wordlists.
- **Interactive TUI:**
  - View and manage your vault entries in an interactive terminal UI.

---

## Installation

### Script
```bash
$ curl https://raw.githubusercontent.com/plu7o/candado/refs/heads/main/install.sh | sh
```

### Manual installation
```bash
# Clone the repository
$ git clone https://github.com/<your-username>/candado.git

# Change directory
$ cd candado

# Build and install
$ cargo build --release
$ cp target/release/candado /usr/local/bin
```
---

## Usage
Candado is divided into two main categories of commands:

1. **Generators** for creating passwords, tokens, keys, or passphrases.
2. **Vault** commands for managing your secure password vault.

```bash
USAGE:
    candado [SUBCOMMAND]

COMMANDS:
    gen      Generate secrets
    vault    Manage passwords
    help     Print help information
    version  Print version information
```

### Examples
#### Generate Secrets
- Generate a password:
  ```bash
  candado gen password -l 16
  ```

- Generate a token:
  ```bash
  candado gen token -l 32
  ```

- Generate a key:
  ```bash
  candado gen key -l 64
  ```

- Generate a passphrase with a custom wordlist:
  ```bash
  candado gen passphrase -l 5 -c /path/to/wordlist.txt
  ```

#### Manage Password Vault
- Initialize a new vault:
  ```bash
  candado vault init
  ```

- Add a new entry:
  ```bash
  candado vault add my-service my-email@example.com -p MySecurePassword -n MyUsername -u https://my-service.com
  ```

- List all entries:
  ```bash
  candado vault ls
  ```

- Find an entry by query:
  ```bash
  candado vault find service-name
  ```

- Update an entry:
  ```bash
  candado vault update entry-id -p NewPassword
  ```

- Remove an entry:
  ```bash
  candado vault rm entry-id
  ```

- Export all entries to a JSON file:
  ```bash
  candado vault export /path/to/backup.json
  ```

- Import entries from a JSON file:
  ```bash
  candado vault import /path/to/backup.json
  ```

---

## Roadmap
- [ ] Extend the TUI interface with more customization options.
- [ ] Build GUI interface for Desktop usage.

---

## How It Works

**Candado** ensures secure management of passwords and secrets through an offline, encrypted vault. Here's a brief overview of its functionality:

1. **Encryption & Security**:
   - Utilizes **AES-GCM** for robust encryption.
   - Vault entries are securely encrypted with a master key derived using **Argon2**, a password-based key derivation function resistant to brute-force attacks.
   - A randomly generated salt and a secure keyfile ensure additional layers of protection.

2. **Password Vault**:
   - Passwords and secrets are stored in an encrypted file, which can only be accessed using the master key.
   - Provides CRUD (Create, Read, Update, Delete) operations on stored entries, enabling seamless management.

3. **Secrets Generator**:
   - Generates strong, customizable passwords, tokens, and passphrases.
   - Ensures high entropy for secure credentials and keys, adhering to modern security standards.

4. **Interactive TUI**:
   - An interactive text-based interface allows you to manage and inspect entries in a user-friendly manner.

5. **Offline-First Design**:
   - Operates entirely offline, ensuring that your sensitive information never leaves your local system.

---

## Contributing
We welcome contributions! If you’d like to contribute to Candado, please follow these steps:

1. Fork the repository.
2. Create a new branch.
3. Make your changes and test thoroughly.
4. Submit a pull request with a detailed description of your changes.

---

## License
This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for more details.

---

## Acknowledgments
- **Rust Community:** For creating an amazing ecosystem for developers.

---

Feel free to open issues or reach out for support. Let’s build secure and modern tools together!

