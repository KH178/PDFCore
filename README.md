# PDFCore

PDFCore is a streaming-first PDF rendering engine written in Rust.

It focuses on **correct PDF structure, deterministic output, and predictable performance at scale**.  
Higher-level authoring (JavaScript, HTML/CSS) is supported through bindings and optional WASM modules, while the Rust core remains the separated.

## Features
- Multi-page PDF generation
- Streaming output with bounded memory usage
- Page-isolated layout and rendering
- Designed for native, Node.js (N-API), and WASM targets

## License
MIT
