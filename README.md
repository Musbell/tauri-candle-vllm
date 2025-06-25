# Message Mate - Professional AI Writing Assistant

Message Mate is a desktop application designed to help users craft clear, precise, and effective communications. Built with Tauri and Solid.js, it runs a local LLM (Large Language Model) to deliver professional writing guidance without requiring an internet connection.

## Features

- **Offline AI Assistance**: Runs a local LLM (Qwen3-4B) to provide efficient writing support without internet dependency
- **Professional Writing Guidance**: Offers assistance with emails, reports, business correspondence, formal letters, announcements, and social media posts
- **User-Friendly Interface**: Intuitive chat interface suitable for users of all technical backgrounds
- **Low Resource Usage**: Optimized for performance on modest hardware

## Installation

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Node.js](https://nodejs.org/) (v16 or later)
- [Bun](https://bun.sh/) package manager

### Installation Steps

1. Clone the repository:
   ```bash
   git https://github.com/Musbell/tauri-candle-vllm
   cd tauri-candle-vllm
   ```

2. Install dependencies:
   ```bash
   bun install
   ```

3. Build the application:
   ```bash
   bun run tauri build
   ```

4. The built application will be available in the `src-tauri/target/release` directory.

## Usage

1. Launch the Message Mate application.
2. Wait for the LLM to initialize (indicated by the status dot turning green).
3. Type your writing request in the input field and press "Send".
4. Receive personalized guidance tailored to your specific communication needs.

## Development

### Setup Development Environment

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/tauri-candle-vllm.git
   cd tauri-candle-vllm
   ```

2. Install dependencies:
   ```bash
   bun install
   ```

3. Start the development server:
   ```bash
   bun run tauri dev
   ```

### Project Structure

- `src/` - Frontend Solid.js code
- `src-tauri/` - Rust backend code
    - `src/lib.rs` - Main Rust code for the Tauri application
    - `bin/candle-vllm` - Sidecar executable for running the LLM

### LLM Configuration

The application uses the Qwen3-4B model with GGUF quantization. The model configuration can be modified in `src-tauri/src/lib.rs`.

## Technical Details

### Frontend

- **Framework**: Solid.js
- **UI**: Custom CSS

### Backend

- **Framework**: Tauri 2.0
- **Language**: Rust
- **LLM Runtime**: Candle (a Rust-native ML framework)
- **Model**: Qwen3-4B (4 billion parameter LLM optimized for professional communication)

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request


## Recommended IDE Setup

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
