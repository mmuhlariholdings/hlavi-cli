# hlavi-cli

Command-line interface for Hlavi kanban task management with AI agent support.

## Table of Contents

- [Getting Started](#getting-started)
- [Documentation](#documentation)
- [Development](#development)
- [Contributing](#contributing)
- [Contact](#contact)

## Getting Started

A quick guide on how you can get started running and working on the applicatoin on your local machine.

### Requirements

- Rust 1.75 or higher
- Cargo

### Clone

```bash
git clone https://github.com/mmuhlariholdings/hlavi-cli.git
cd hlavi-cli
```

### Install

```bash
cargo install --path .
```

### Usage

Initialize a project:
```bash
hlavi init
```

Create a ticket:
```bash
hlavi tickets create "Implement user authentication"
```

List tickets:
```bash
hlavi tickets list
```

Edit a ticket:
```bash
hlavi tickets edit HLA1 -d "Add JWT-based authentication"
hlavi tickets edit HLA1 --ac "User can log in with email and password"
```

View ticket details:
```bash
hlavi tickets show HLA1
```

## Commands

- `hlavi init` - Initialize a Hlavi project
- `hlavi tickets list` - List all tickets
- `hlavi tickets create <title>` - Create a new ticket
- `hlavi tickets edit <id>` - Edit a ticket
- `hlavi tickets show <id>` - Show ticket details
- `hlavi tickets delete <id>` - Delete a ticket
- `hlavi board show` - View the kanban board (coming soon)
- `hlavi agent configure` - Configure AI agent (coming soon)

## Development

During development, use `cargo run` instead of installing the CLI every time. This is much faster and allows for quick iteration.

```bash
# Run the CLI directly (no installation needed)
cargo run -- init
cargo run -- tickets
cargo run -- tickets create "Test ticket"
cargo run -- tickets show HLA1
```

### Testing

Run tests to validate your changes:

```bash
# Run all tests
cargo test

# Run tests with verbose output
cargo test -- --nocapture

# Run a specific test
cargo test test_name
```

### When to Install

Only install when you need to:

1. **Test the final user experience:**
   ```bash
   cargo install --path .
   hlavi tickets
   ```

2. **Validate before release** - Ensure the installed version works correctly

3. **Use it for real work** - When actually using hlavi to manage tasks

## Contributing

Take a moment to review our [contribution guide](CONTRIBUTING.md) before submitting your first pull request.

Make sure that you check for open issues and pull requests to see if someone else is working on something similar.

## Contact

For feedback, requests or enquiries:

🌐 [http://www.mmuhlariholdings.co.za](http://www.mmuhlariholdings.co.za)