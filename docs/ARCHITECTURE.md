# vendrtk — Architecture

## What is this?

A Rust library that takes a PDF (and most likely more input formats in the future), either a vendor invoice or a Statement of Work (or neither), chews through it, and spits out clean structured output. It does this via a small and flexible pipeline of independent library crates, stitched together by a thin facade, in an idempotent manner. so reprocessing is cheap and the whole thing plays nicely in a serverless environment.

## Workspace

`vendrtk-core` is the facade. It re-exposes the independent pipeline crates as a single clean API so binaries have one thing to import and all the tools they need right there. Concrete implementation details — web server config, CLI flags, credentials, tracing setup — live in the binaries. The libs don't know about any of that.

Nothing leaks upward. Public API is clean from day one.

## Planned Layout

```
vendrtk/
├── Cargo.toml                  # workspace manifest
├── Makefile
├── crates/
│   ├── bin/
│   │   └── webapp/             # axum web server
│   └── lib/
│       ├── vendrtk/       # facade — re-exposes libs, possibly gates features behind feature flags
│       ├── ocr/           # talks to the OCR backend, nothing else (Azure Doc Intel to Start)
│       ├── parsers/       # extracts structured fields from OCR output
│       ├── stores/        # persistence — OCR results, parsed records
│       ├── pipelines/     # orchestration
│       ├── azure/         # azure specific, i.e. models for responses from DocIntel, auth, etc.
│       └── ...            # more crates as the pipeline grows
├── docs/
│   └── ARCHITECTURE.md         # you are here
└── target/
```

---

## Pipeline

The pipeline is intentionally idempotent. Expensive stages must be skipped if their output is already stored — no re-running OCR on a file we've seen before, no re-parsing a record that's already "clean".

```
File upload                                       (Uploaded File Store, metadata only)?
    |
    v
stores              load bytes, detect format, fingerprint the file
    |
    v
ocr                 submit to OCR backend         ← skippable if fingerprint hit in OCR store
    |
    v
stores              persist OCR result            (OCR Store)
    |
    v
parsers             extract structured fields     ← skippable if already parsed
    |
    v
validators          validate parsed output
    |
    v
stores              persist parsed record         (Parser Store)
    |
    v
                    emit structured output        ← CSV? Parquet? TBD
```

---

## Crates

### `ocr`
Submits a document to the OCR backend (Azure Doc Intel to start, will attempt to make this generic enough so other providers could be included in the future) and returns structured OCR output.

### `parsers`
Takes OCR output + Original File bytes. Contains parsers, and extracts structured fields, regex for the well-defined stuff, deterministic layout logic for known vendor formats, LLM as a baseline parser best effor attempt. Returns typed records.

### `stores`
Persistence layer. Stores OCR results and parsed records keyed by file fingerprint. This is what makes the pipeline idempotent — hit the store before running the expensive stage. Will start with local stores (probably json ledger and file combo, will determine if compatible with SQL Storage implementations or anything ACID, I want this to be agnostic, able to run locally using files, or large scale, with any SQL DB that supports Jsonb).

### `pipelines`
Simple pipeline declaration for prebuilt flows.

### `vendrtk`
The facade. Re-exports what the binaries need and wires the pipeline together. May use feature flags to gate optional capabilities. Binaries import this and nothing else to keep a clean API.

### `webapp` (bin)
For now the only concrete implementation. It'll be an axum web server (frontend could be Yew, perhaps Leptos, will define later), intended to run as a standalone container. Will handle file uploads, drives the pipeline via `vendrtk-core`, returns results. Owns all runtime concerns, tracing spans, config, ports, credentials. Contains zero business logic.

---

## Output format

CSV is the current assumption. Parquet is on the table for larger volumes. TBD, but the pipeline emits typed records and the serialization step is intentionally last and swappable.

---

## Dependencies — keep it lean

No heavy SDKs for external service providers. Thin clients only.

| What | How |
|---|---|
| Async runtime | `tokio` |
| Web framework | `axum` |
| HTTP client | `reqwest` |
| Serialization | `serde` + `serde_json` |
| LLM structured extraction | `rig` |
| Observability | `tracing` + `tracing-subscriber` |
| Will need to take a look at Azure crates | `azure*` |

## Wishlist

User defined schemas.
Structured Deidentification.
Canonical Names for Vendors, Services, etc. Outside the scope but some ideas that come to mind: NLP based entity recognition / Perhaps Transformer based, will look into this later.