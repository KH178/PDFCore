Here’s a **clean, ready-to-drop `spec.md`** for your repo.
You can copy-paste this directly into `spec.md` without changes.

---

```md
# PDFX — Project Specification

## 1. Overview

**PDFX** is a high-performance, streaming-first PDF generation engine with a Rust core and a JavaScript API.  
It is designed for large-scale document generation with predictable performance, bounded memory usage, and a developer-friendly declarative API inspired by HTML layout models.

PDFX is an **engine**, not a wrapper around a browser or external renderer.

---

## 2. Core Goals

### Primary Goals
- Generate PDFs efficiently at scale (10,000 pages < 5 minutes on commodity hardware)
- Support streaming output without buffering the full document in memory
- Enable safe multithreaded rendering
- Provide deterministic, repeatable output
- Expose a clean, easy-to-use JavaScript API

### Non-Goals (v1)
- Full HTML/CSS rendering
- Browser-level layout fidelity
- Interactive PDFs (forms, embedded JS)
- WYSIWYG document editing

---

## 3. Target Use Cases

- Invoices and billing documents
- Financial and analytical reports
- SaaS data exports
- Compliance and audit documents
- Internal automation tools

---

## 4. Architecture Overview

```

JavaScript / TypeScript API
↓
N-API / WASM Bindings
↓
Layout Frontend (HTML/CSS-inspired)
↓
Rust Core Engine
├─ Document Model
├─ Layout Engine
├─ Text & Font System
├─ Renderer
├─ Streaming Writer
└─ Output Backend


````

---

## 5. Technology Choices

| Layer | Technology | Rationale |
|-----|-----------|-----------|
| Core Engine | Rust | Memory safety, performance, concurrency |
| JS Bindings | napi-rs | Stable ABI, zero-copy buffers |
| Build System | Cargo | Deterministic builds |
| Fonts | HarfBuzz + FreeType | Industry-standard text shaping |
| Compression | flate2 (zlib) | PDF stream compression |
| Concurrency | Rayon + channels | Safe parallelism |

---

## 6. Design Principles

1. **Streaming-First**
   - Never require the entire document in memory
   - Page-level buffering only

2. **Page Isolation**
   - Pages render independently
   - Enables parallel layout and rendering

3. **Deterministic Output**
   - Same input always produces identical output bytes

4. **Backpressure-Aware**
   - Bounded queues
   - Prevents uncontrolled memory growth

---

## 7. PDF Feature Scope

### Supported (v1)
- PDF 1.7
- Single and multi-page documents
- Text (Latin scripts initially)
- Built-in fonts (Helvetica, Times)
- Absolute positioning
- Standard page sizes (A4, Letter)
- Stream compression

### Planned (v2+)
- Unicode shaping
- Custom fonts (TTF/OTF)
- Images (JPEG/PNG)
- Tables
- Automatic pagination
- Headers and footers

---

## 8. Internal Data Model

### Document
```rust
struct Document {
    pages: Vec<Page>,
    resources: Resources,
}
````

### Page

```rust
struct Page {
    width: f32,
    height: f32,
    content: Vec<RenderOp>,
}
```

### Render Operations

```rust
enum RenderOp {
    Text(TextOp),
    Image(ImageOp),
    Block(BlockOp),
}
```

---

## 9. Layout Engine

### Layout Model

* Vertical flow layout (top → bottom)
* No floats or CSS positioning
* Explicit pagination rules

### Algorithm

1. Measure content
2. Check available page space
3. Insert page break if needed
4. Render block

This model is LaTeX-style, not browser-style.

---

## 10. Text Rendering Pipeline

```
Text
 → UTF-8 Decode
 → Shaping (HarfBuzz)
 → Glyph Runs
 → Font Metrics
 → PDF Text Commands
```

### Constraints

* No implicit font fallback in v1
* Fonts must be explicitly specified

---

## 11. Streaming & Output Strategy

### Output Model

* Each page renders into an isolated buffer
* Final writer stitches pages sequentially
* Cross-reference table generated at the end

### Writer Responsibilities

* Track object offsets
* Write objects sequentially
* Emit xref and trailer
* Flush output incrementally

---

## 12. Concurrency Model

### Parallelism Unit

* Page-level parallelism

### Pipeline

```
JS Producer
   ↓
Layout Workers (Rayon)
   ↓
Render Workers
   ↓
Single Writer Thread
```

### Guarantees

* No shared mutable state across threads
* Deterministic output ordering
* Bounded memory usage

---

## 13. Performance Targets

| Metric       | Target       |
| ------------ | ------------ |
| Single page  | < 5 ms       |
| 1,000 pages  | < 30 seconds |
| 10,000 pages | < 5 minutes  |
| Peak RAM     | < 300 MB     |
| Startup time | < 50 ms      |

---

## 14. JavaScript API (Planned)

```ts
const doc = new PDFX();

doc.page()
   .text("Invoice", { size: 24 })
   .text("Customer: John");

await doc.streamTo("out.pdf");
```

### API Characteristics

* Declarative
* Chainable
* Streaming-friendly
* Async-first

---

## 15. Error Handling & Safety

* No panics across FFI boundaries
* Structured error codes
* Partial output cleanup on failure
* Temp-file + atomic rename strategy

---

## 16. Testing Strategy

### Unit Tests

* PDF object serialization
* Offset tracking
* Page rendering correctness

### Integration Tests

* Open generated PDFs in:

  * Adobe Reader
  * Chrome PDF Viewer

### Benchmarks

* Stress tests for 1k / 5k / 10k pages

---

## 17. Versioning Roadmap

| Version | Scope              |
| ------- | ------------------ |
| v0.1    | Single-page PDF    |
| v0.2    | Multi-page support |
| v0.3    | Streaming writer   |
| v0.4    | Fonts & shaping    |
| v1.0    | Production-ready   |

---

## 18. Stage-Gated Development Policy

### Principle

The project advances **strictly one stage at a time**.
No work from a future stage may begin until the current stage is fully complete and validated.

---

## 19. Definition of Done (DoD)

A stage is considered complete only when:

### Functional

* All scoped features implemented
* PDFs open correctly in Adobe Reader and Chrome

### Technical

* No TODOs or commented-out logic
* No panics in normal execution
* Errors handled explicitly

### Performance

* Stage performance targets met
* No unbounded memory usage

### Validation

* Manual test executed
* Automated test added
* Benchmark recorded

---

## 20. Stage Progression Rules

### Allowed

* Refactoring within the current stage
* Bug fixes
* Internal API cleanup

### Not Allowed

* Implementing future-stage features early
* Premature optimization
* Expanding public APIs without use

---

## 21. Stage Completion Checklist

```
[ ] Feature complete
[ ] Manual PDF verification (Adobe + Chrome)
[ ] Benchmarked
[ ] Tests added
[ ] Code cleaned
[ ] Stage documented
```

If any item is unchecked, the project **must not advance**.

---

## 22. Current Active Stage

### v0.1 — Single Page PDF

Scope:

* One page
* One font
* One content stream
* Correct cross-reference table
* Correct trailer

Explicitly excluded until v0.1 is complete:

* Multiple pages
* Layout engine
* Multithreading
* Streaming APIs
* JavaScript bindings

---

## 23. Stage Transition Rule


## 24. WASM & HTML/CSS Integration 

PDFX will provide an optional WebAssembly (WASM) module that exposes the core engine to web and JavaScript environments.

### Goals
- Reuse the same Rust core for native and web targets
- Enable HTML/CSS-inspired document authoring
- Avoid browser engines or DOM-based rendering

### Architecture
- Rust core compiled to WASM
- WASM module exposes a layout and rendering API
- HTML/CSS parsed into an intermediate layout tree
- Layout tree mapped to PDFX’s internal document model

HTML and CSS act as an **authoring frontend**, not the rendering engine.

### Non-Goals
- Pixel-perfect browser fidelity
- DOM execution or JavaScript inside WASM
- Dependency on Chromium, WebKit, or Gecko

**The project moves to the next stage only when the current stage is correct, performant, and boring.**

