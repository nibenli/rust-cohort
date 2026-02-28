```markdown
# Rust JSON Parser

A high-performance JSON parser built with Rust and exposed as a native Python module using **PyO3** and **Maturin**. This project demonstrates a "mixed" layout, combining Rust's safety and speed with Python's ease of use.

## 🚀 Environment Setup

Follow these steps to set up your development environment from scratch.

### 1. Prerequisites
* **Rust**: [Install Rust](https://rustup.rs/) (includes `cargo`).
* **Python**: Version 3.12 or higher.

### 2. Create a Virtual Environment
```bash
python3 -m venv .venv
source .venv/bin/activate

```

### 3. Install Python Dependencies

Use the provided `requirements.txt` to install `maturin` (for building the Rust extension) and `pytest` (for testing).

```bash
pip install -r requirements.txt

```

---

## 🛠 Building and Running

This project uses a `Makefile` to automate the "bridge" between Rust and Python.

### Important: The First Step

Before you can import the library or run the CLI, you must compile the Rust code and install the module into your virtual environment:

```bash
make develop

```

*Note: You must re-run this command whenever you modify the Rust source code.*

---

## 🧪 Testing

The project includes both Rust unit tests and Python integration tests to ensure the parser behaves correctly across the boundary.

| Command | Description |
| --- | --- |
| `make test` | Runs **both** Rust and Python test suites. |
| `make test-rust` | Runs Rust unit tests (`cargo test --lib`). |
| `make test-python` | Runs Python integration tests (`pytest -v`). |

---

## 💻 CLI Usage Examples

The package includes a CLI wrapper that can be invoked via `python -m`. The `Makefile` provides quick shortcuts to test different input types:

* **Parse a File**:
```bash
make run-file

```


* **Parse a String**:
```bash
make run-string

```


* **Parse from Stdin (Piping)**:
```bash
make run-pipe

```



---

## 📂 Project Structure

```text
.
├── Cargo.toml            # Rust metadata and dependencies
├── pyproject.toml        # Python build-system configuration (Maturin)
├── requirements.txt      # Python development dependencies
├── Makefile              # Automation shortcuts
├── src/                  # Rust source code
│   └── lib.rs            # PyO3 bindings and module logic
├── python/               # Python source code
│   └── rust_json_parser/ 
│       ├── __init__.py   # Package entry point
│       └── __main__.py   # CLI logic
└── tests/                # Python integration tests

```

## 🧹 Cleanup

To remove all build artifacts, compiled binaries, and temporary files:

```bash
make clean

```