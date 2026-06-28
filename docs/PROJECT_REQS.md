# Product Requirements Document
## Invoice & SOW Parsing Service — Rust Rewrite

---

### 1. Work Context

**Team:** Internal tooling team  
**Business Need:** The company processes vendor invoices and Statements of Work (SOWs) for reconciliation purposes. The current Python-based pipeline ingests PDFs via SharePoint + Power Automate, parses them, and syncs results back. Error prone, little observability, hard to extend. The goal is to replace this with a lean, self-contained Rust service with improved observability, security posture, and deployment footprint.

---

### 2. Problem Statement

The existing Python solution has three core issues:

- **Supply chain risk:** A large number of third-party dependencies increases exposure to software supply chain vulnerabilities.
- **Poor observability:** Error management and tracing are insufficient to pinpoint failures within the pipeline stages.
- **Heavy runtime:** Python's ecosystem overhead makes it costly and cumbersome to run as a cloud function or small container. Current implementation runs in serverless function, adding cold start.

---

### 3. Goals & Success Criteria

| Goal | Success Criteria |
|------|-----------------|
| Correct document classification | PDFs are reliably identified as Invoice, SOW, or Unknown |
| Accurate invoice field extraction | All required invoice fields extracted with high fidelity |
| Accurate SOW field extraction | All required SOW fields extracted on a best-effort basis |
| Structured output | Results exported as well-formed CSV |
| Observability | Per-stage tracing allows pinpointing failures in the pipeline |
| Deployment footprint | Runs as a single small container; no heavy runtime dependency |
| Improved UX | Internal users upload files and inspect results via a lightweight web UI |

---

### 4. Core Deliverables

1. **PDF Ingestion & Classification** — Accept a PDF upload, detect document type (Invoice / SOW / Unknown) using deterministic heuristics and/or LLM.
2. **OCR Integration** — Hand-rolled HTTP client using [reqwest](https://github.com/seanmonstar/reqwest) targeting the [Azure Document Intelligence](https://learn.microsoft.com/en-us/azure/ai-services/document-intelligence/) REST API.
3. **Field Extraction Pipeline** — Parse OCR output into structured records using a combination of:
   - Regex for well-defined fields (dates, invoice numbers)
   - Deterministic logic for known layouts
   - [rig](https://github.com/0xPlaygrounds/rig) for LLM-powered structured extraction of ambiguous or complex fields (typed schemas via `serde`)
4. **CSV Export** — Serialize extracted records into CSV with clearly defined schemas per document type using [csv](https://github.com/BurntSushi/rust-csv).
5. **Lightweight Web UI** — File upload interface and pipeline inspection view built on [axum](https://github.com/tokio-rs/axum).

---

### 5. Field Schemas

#### Invoice Fields
| Field | Type |
|-------|------|
| `vendor` | String |
| `invoice_number` | String |
| `invoice_date` | Date |
| `invoice_facility` | String |

#### Invoice Line Items
| Field | Type |
|-------|------|
| `service_name` | String |
| `service_start_date` | Date |
| `service_end_date` | Date |
| `service_line` | String |

#### SOW Fields
| Field | Type |
|-------|------|
| `vendor` | String |
| `valid_from` | Date |
| `valid_to` | Date |
| *(service rates and additional contractual fields — best effort)* | TBD |

---

### 6. Scope & Constraints

#### In Scope (6 weeks)
- PDF upload and document type classification
- Azure Document Intelligence OCR integration (custom [reqwest](https://github.com/seanmonstar/reqwest) client)
- Invoice parsing pipeline → CSV export
- SOW parsing pipeline → CSV export (best effort)
- Lightweight web UI for upload and result inspection
- Structured per-stage tracing and error handling

#### Out of Scope / Deferred
- SOW ↔ Invoice reconciliation logic (future work; noted: will require name canonicalization strategy)
- SharePoint / Power Automate integration (replaced by web UI)
- Authentication / access control (internal tool, first iteration)
- Bulk batch processing

#### Constraints
- **Language:** Rust (stable)
- **OCR:** [Azure Document Intelligence](https://learn.microsoft.com/en-us/azure/ai-services/document-intelligence/) — custom REST client via [reqwest](https://github.com/seanmonstar/reqwest)
- **LLM:** OpenAI via [rig](https://github.com/0xPlaygrounds/rig) for structured field extraction
- **Web framework:** [axum](https://github.com/tokio-rs/axum)
- **Async runtime:** [tokio](https://github.com/tokio-rs/tokio)
- **Observability:** [tracing](https://github.com/tokio-rs/tracing) with structured spans per pipeline stage
- **CSV output:** [csv](https://github.com/BurntSushi/rust-csv) crate
- **Serialization:** [serde](https://github.com/serde-rs/serde) for typed field schemas
- **Dependency philosophy:** Minimize third-party crates; prefer audited, well-maintained crates; build thin clients over pulling large SDKs
- **Deployment:** Single Docker container; no heavy runtime

---

### 7. Suggested 6-Week Milestones

| Week | Focus |
|------|-------|
| 1 | Project scaffold: axum server, file upload endpoint, tracing setup |
| 2 | Azure Document Intelligence HTTP client + OCR response modeling |
| 3 | Document classification logic (Invoice / SOW / Unknown) |
| 4 | Invoice field extraction pipeline + CSV export |
| 5 | SOW field extraction pipeline + CSV export |
| 6 | Web UI polish, error handling hardening, containerization |