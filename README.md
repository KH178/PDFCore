Here is the updated minimal README with proper data streaming example added and aligned with your spec.

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
* SQLite cursor-based data streaming
* Batch data stream rendering
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

## Streaming Data Example

PDFCore can render large datasets without loading them fully into memory.

```js
const fs = require("fs");
const { Readable } = require("stream");
const { PDFCoreClient } = require("@pdfcore/client");

const readable = new Readable({
  read() {}
});

for (let i = 0; i < 1000; i++) {
  readable.push(JSON.stringify({ counter: i }) + "\n");
}
readable.push(null);

const client = new PDFCoreClient();

await client.renderBatch({
  template: "stream_template.pdfCoret",
  dataStream: readable,
  output: "stream_output.pdf"
});
```

This ensures:

* Incremental row processing
* Constant memory usage
* Page-level buffering only
* Deterministic output

---

## CLI Example

```bash
pdfcore render invoice.pdfCoret --data data.json --out invoice.pdf
```

Streaming from SQLite:

```bash
pdfcore render report.pdfCoret --db ./data.db --param 101 --out report.pdf
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
