# Mystera Legacy Bot

## General info

Mystera reborn is a mod project for the game Mystera Legacy.

## Website

- [Mystera Reborn](https://pimentelm.github.io/mystera-reborn/)

## How to Use (Rust Version)

This project has been partially rewritten in Rust. The server and the core bot logic are now in Rust, while the frontend remains in TypeScript/Vue.js.

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install)
- [`wasm-pack`](https://rustwasm.github.io/wasm-pack/installer/)
- [Node.js](https://nodejs.org/) (for the frontend)

### Building the Project

The project is split into two main parts: the Rust backend/WASM and the TypeScript frontend.

**1. Build the Rust Server**

The server is an Axum web server that serves the frontend and proxies WebSocket traffic.

```sh
cd rust
cargo build --release
```
The binary will be located at `rust/target/release/server`.

**2. Build the WASM Bot Logic**

The bot's core logic is compiled to WebAssembly.

```sh
cd rust/bot-logic
wasm-pack build --target web --out-dir pkg
```
This will create a `pkg` directory with the WASM module and JavaScript bindings.

**3. Build the Frontend**

***Note: The frontend dependencies are currently outdated and may not install or build correctly in a modern Node.js environment.***

If the dependencies can be installed, the build process is:

```sh
cd frontend
npm install
npm run build
```
This will build the frontend assets and place them in the `public/` directory at the project root.

### Running the Application

Once all the components are built, you can run the application by starting the Rust server.

```sh
./rust/target/release/server
```

The server will be available at `http://127.0.0.1:3000`.
