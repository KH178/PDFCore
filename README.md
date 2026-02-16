Understood. Here is a clean, neutral, minimal README without attitude and without using double hyphens.

---

# PDFCore

PDFCore is a streaming-first PDF generation engine built in Rust with a JavaScript API.

It is designed for deterministic, large-scale document generation with bounded memory usage and a template-driven workflow.

PDFCore is an engine, not a browser wrapper.

---

## Features

* Streaming-first architecture
* Deterministic output
* Page-level parallel rendering
* Bounded memory usage
* Custom `.pdfCoret` template format
* SQLite streaming data binding
* Asset deduplication
* Multi-page document support
* CLI and JavaScript SDK

---

## Architecture

```
Editor
   ↓
Template (.pdfCoret)
   ↓
CLI or JS SDK
   ↓
Rust Core Engine
   ↓
Streaming Writer
```

No HTML rendering
No DOM dependency
No browser engine

---

## JavaScript Example

```ts
import { PDFCoreClient } from "@pdfcore/client";

const client = new PDFCoreClient();

await client.render({
  template: "invoice.pdfCoret",
  data: invoiceData,
  output: "invoice.pdf"
});
```

---

## CLI Example

```bash
pdfcore render invoice.pdfCoret --data data.json --out invoice.pdf
```

---

## Template Format

```
template.pdfCoret
├── layout.json
├── styles.json
├── assets/
└── manifest.json
```

Templates are versioned, deterministic, and portable.

---

## License

MIT
