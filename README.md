# Llama on Rust

A web interface for interacting with open-source LLMs powered by Rust and leveraging [mistral.rs](https://github.com/EricLBuehler/mistral.rs) as the backend server.

## Overview

This project provides a simple, lightweight web application that allows you to chat with local LLMs through a clean interface. It's designed to be flexible and supports various model formats including GGUF and Hugging Face models.

This project was inspired by [MoonKraken's](https://github.com/MoonKraken) [rusty_llama](https://github.com/MoonKraken/rusty_llama) repo. I wanted to implement a similar project for practice, but the Rust llm libraries used in his project are no longer being maintained, hence the usage of mistralrs.

## Features

- Support for both GGUF/GGML and Hugging Face models via mistral.rs
- Configurable parameters (temperature, top-p, max tokens)=

## Prerequisites

- [Rust](https://rustup.rs/)
- A compatible LLM model file (GGUF format recommended)
- [mistral.rs](https://github.com/EricLBuehler/mistral.rs) server running locally

## Quick Start

1. Clone this repository:
```bash
git clone https://github.com/ericgulottyjr/llama-on-rust.git
cd llama-on-rust
```

2. Create a `.env` file with your configuration (optional, these are default values):
```
MISTRAL_SERVER_URL=http://localhost:8081
RUST_LOG=info
TEMPERATURE=0.7
TOP_P=0.95
MAX_TOKENS=512 
```

3. Start the mistral.rs server:
```bash
# Install and run mistral.rs server
git clone https://github.com/EricLBuehler/mistral.rs.git
cd mistral.rs
cargo build --release --features metal  # For macOS, or use cuda for NVIDIA GPUs
./target/release/mistralrs-server --port 8081 gguf -m /path/to/model/directory -f your-model.gguf
```

4. Build and run the web application:
```bash
cargo build --release
cargo run --release
```

5. Open your browser and navigate to http://localhost:8080

## Architecture

The application consists of three main components:

1. **Web Interface**: Handles HTTP requests, renders templates, and manages user sessions
2. **Model Interface**: Communicates with the mistral.rs server via its API
3. **Session Manager**: Maintains conversation history and context

## API Endpoints

- `GET /` - Web interface
- `GET /health` - Health check endpoint
- `POST /api/chat` - Chat endpoint
  - Request: `{ "message": "Your message", "session_id": "optional-uuid", "max_tokens": 100 }`
  - Response: `{ "response": "Model response", "session_id": "uuid" }`

## Future Work

Some work that I hope to complete in the future:

- **User Interface Enhancements** (markdown rendering, syntax highlighting, dark/light theme)
- **Model Management** (model switching, parameter presets, custom prompts)
- **Deployment Features** (resource monitoring dashbaord)

## Troubleshooting

- **Model Loading Issues**: Ensure your model path is correct and the model is in a supported format
- **Performance Problems**: Adjust the model size or inference parameters in your .env file
- **Connectivity Issues**: Verify that the mistral.rs server is running and accessible at the configured URL

## Acknowledgements

- [EricLBuehler](https://github.com/EricLBuehler) for creating mistral.rs
- [MoonKraken](https://github.com/MoonKraken) for rusty-llama
- The entire Rust community
