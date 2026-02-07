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

Information on how to go about your development workflow.

## Contributing

Take a moment to review our [contribution guide](CONTRIBUTING.md) before submitting your first pull request.

Make sure that you check for open issues and pull requests to see if someone else is working on something similar.

## Contact

For feedback, requests or enquiries:

🌐 [http://www.mmuhlariholdings.co.za](http://www.mmuhlariholdings.co.za)