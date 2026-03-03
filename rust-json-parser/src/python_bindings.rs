use crate::{JsonError, JsonParser, JsonValue};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyList, PyTuple};
use pyo3::{Bound, IntoPyObject, PyAny, PyErr};
use std::collections::HashMap;
use std::time::Instant;

impl<'py> IntoPyObject<'py> for JsonValue {
    type Target = PyAny;
    type Output = Bound<'py, Self::Target>;
    type Error = PyErr;

    fn into_pyobject(self, py: Python<'py>) -> Result<Self::Output, Self::Error> {
        match self {
            // Null maps to Python's None
            JsonValue::Null => Ok(py.None().into_bound(py)),

            // Primitives: bool and f64 implement IntoPyObject themselves.
            JsonValue::Boolean(b) => Ok(b.into_pyobject(py)?.to_owned().into_any()),
            JsonValue::Number(n) => Ok(n.into_pyobject(py)?.to_owned().into_any()),

            // Strings: Convert to Python string then cast to Any
            JsonValue::String(s) => Ok(s.into_pyobject(py)?.into_any()),

            // Arrays: Recursively convert elements and append to a PyList
            JsonValue::Array(arr) => {
                let list = PyList::empty(py);
                for item in arr {
                    list.append(item.into_pyobject(py)?)?;
                }
                Ok(list.into_any())
            }

            // Objects: Recursively convert keys and values into a PyDict
            JsonValue::Object(obj) => {
                let dict = PyDict::new(py);
                for (key, val) in obj {
                    dict.set_item(key.into_pyobject(py)?, val.into_pyobject(py)?)?;
                }
                Ok(dict.into_any())
            }
        }
    }
}

impl From<JsonError> for PyErr {
    fn from(err: JsonError) -> PyErr {
        // Use Display implementation for JsonError to create the Python error message.
        // This ensures the "at position X" info is preserved.
        PyValueError::new_err(err.to_string())
    }
}

/// Parses a JSON string into a Python object.
///
/// This function takes a raw JSON string, parses it using the internal Rust engine,
/// and converts the resulting `JsonValue` tree into native Python types
/// (e.g., `dict`, `list`, `str`, `float`, `bool`, or `None`).
///
/// # Examples
///
/// ```python
/// import rust_json_parser
///
/// data = rust_json_parser.parse_json('{"name": "Alice", "active": true}')
/// print(data["name"]) # Output: Alice
/// ```
///
/// # Errors
///
/// Returns a `ValueError` if the input string is not valid JSON. The error
/// message will include the specific reason for the failure and the
/// byte position where the error was detected.
#[pyfunction]
pub fn parse_json<'py>(py: Python<'py>, input: &str) -> PyResult<Bound<'py, PyAny>> {
    let json_value = JsonParser::new(input)?.parse()?;

    let py_object = json_value.into_pyobject(py)?;

    Ok(py_object)
}

/// Reads a file and parses its contents as JSON.
///
/// This is a convenience function that handles reading the file from the
/// filesystem and immediately parsing it. It uses UTF-8 encoding by default.
///
/// # Examples
///
/// ```no_run
/// import rust_json_parser
///
/// try:
///     testdata = rust_json_parser.parse_json_file("testdata.json")
/// except FileNotFoundError:
///     print("testdata file missing!")
/// ```
///
/// # Errors
///
/// This function can return several types of errors:
/// * `IOError` (or `FileNotFoundError`): If the file cannot be found, opened, or read.
/// * `ValueError`: If the file exists but contains invalid JSON syntax.
#[pyfunction]
pub fn parse_json_file<'py>(py: Python<'py>, path: &str) -> PyResult<Bound<'py, PyAny>> {
    // The '?' operator here handles std::io::Error.
    // PyO3 automatically converts this to a Python IOError/FileNotFoundError.
    let contents = std::fs::read_to_string(path)?;

    let json_value = JsonParser::new(&contents)?.parse()?;

    let py_object = json_value.into_pyobject(py)?;

    Ok(py_object)
}

pub fn py_to_json_value(obj: &Bound<PyAny>) -> PyResult<JsonValue> {
    // Check for None (Python Null)
    if obj.is_none() {
        return Ok(JsonValue::Null);
    }
    // Check for Boolean (MUST come before Number/f64)
    if let Some(val) = try_bool(obj) {
        return Ok(val);
    }
    // Check for Number (f64)
    if let Some(val) = try_number(obj) {
        return Ok(val);
    }
    // Check for String
    if let Some(val) = try_string(obj) {
        return Ok(val);
    }
    // Check for List (recurse on elements)
    if let Some(val) = try_array(obj)? {
        return Ok(val);
    }
    // Check for Dictionary (recurse on values)
    if let Some(val) = try_object(obj)? {
        return Ok(val);
    }

    // Unsupported type
    Err(PyValueError::new_err(format!(
        "Unsupported type {} for JSON conversion",
        obj.get_type()
    )))
}

#[pyfunction]
#[pyo3(signature = (obj, indent=None))]
pub fn dumps(obj: Bound<PyAny>, indent: Option<usize>) -> PyResult<String> {
    // 1. Convert Python object to our Rust JsonValue enum
    let json_value = py_to_json_value(&obj)?;

    // 2. Format based on indentation
    match indent {
        // Compact mode: use our Display implementation
        None => Ok(json_value.to_string()),

        // Pretty-print mode: use a recursive formatter
        Some(n) => Ok(json_value.pretty_print(n)),
    }
}

#[pyfunction]
pub fn generate_json_with_size(size: usize) -> String {
    let mut obj = String::from("{");
    let mut i = 0;
    while obj.len() < size - 10 {
        // Leave room for closing
        if i > 0 {
            obj.push_str(", ");
        }
        obj.push_str(&format!("\"key_{i}\": {i}"));
        i += 1;
    }
    obj.push('}');
    obj
}

#[pyfunction]
pub fn generate_nested_json(depth: usize) -> String {
    let mut s = String::with_capacity(depth * 10);
    for _ in 0..depth {
        s.push_str("{\"a\":");
    }
    s.push('1');
    for _ in 0..depth {
        s.push('}');
    }
    s
}

#[pyfunction]
#[pyo3(signature = (json_str, iterations=1000))]
pub fn benchmark_performance(
    py: Python<'_>,
    json_str: &str,
    iterations: usize,
) -> PyResult<(f64, f64, f64)> {
    // Import Python modules and prepare arguments
    let json_module = py.import("json")?;
    let loads_fn = json_module.getattr("loads")?;
    let simplejson_module = py.import("simplejson")?;
    let simple_loads_fn = simplejson_module.getattr("loads")?;
    let args = PyTuple::new(py, [json_str])?;

    // --- Warmup Phase ---
    // Ensure all engines and caches are hot (10% of iterations, min 10)
    let warmup = (iterations / 10).max(10);
    for _ in 0..warmup {
        let _ = JsonParser::new(json_str)?.parse()?;
        let _ = loads_fn.call1(&args)?;
        let _ = simple_loads_fn.call1(&args)?;
    }

    // Helper for Median
    let calculate_median = |mut times: Vec<f64>| -> f64 {
        times.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mid = times.len() / 2;
        if times.len().is_multiple_of(2) {
            (times[mid - 1] + times[mid]) / 2.0
        } else {
            times[mid]
        }
    };

    // 1. Benchmark Rust
    let mut rust_times = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = JsonParser::new(json_str)?.parse()?;
        rust_times.push(start.elapsed().as_secs_f64());
    }

    // 2. Benchmark Python Built-in (C-implementation)
    let mut py_times = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = loads_fn.call1(&args)?;
        py_times.push(start.elapsed().as_secs_f64());
    }

    // 3. Benchmark SimpleJSON (Pure Python)
    let mut simple_times = Vec::with_capacity(iterations);
    for _ in 0..iterations {
        let start = Instant::now();
        let _ = simple_loads_fn.call1(&args)?;
        simple_times.push(start.elapsed().as_secs_f64());
    }

    Ok((
        calculate_median(rust_times),
        calculate_median(py_times),
        calculate_median(simple_times),
    ))
}

// Module registration
#[pymodule]
fn _rust_json_parser(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_json, m)?)?;
    m.add_function(wrap_pyfunction!(parse_json_file, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    m.add_function(wrap_pyfunction!(generate_json_with_size, m)?)?;
    m.add_function(wrap_pyfunction!(generate_nested_json, m)?)?;
    m.add_function(wrap_pyfunction!(benchmark_performance, m)?)?;
    m.add("__version__", "0.1.0")?;
    Ok(())
}

// --- Helper Functions ---
fn try_bool(obj: &Bound<PyAny>) -> Option<JsonValue> {
    // check type explicitly because extract::<bool> can be loose
    if obj.is_instance_of::<PyBool>() {
        obj.extract::<bool>().ok().map(JsonValue::Boolean)
    } else {
        None
    }
}

fn try_number(obj: &Bound<PyAny>) -> Option<JsonValue> {
    // handles both ints and floats
    obj.extract::<f64>().ok().map(JsonValue::Number)
}

fn try_string(obj: &Bound<PyAny>) -> Option<JsonValue> {
    obj.extract::<String>().ok().map(JsonValue::String)
}

fn try_array(obj: &Bound<PyAny>) -> PyResult<Option<JsonValue>> {
    if let Ok(list) = obj.cast::<PyList>() {
        let mut arr = Vec::with_capacity(list.len());
        for item in list.iter() {
            arr.push(py_to_json_value(&item)?);
        }
        Ok(Some(JsonValue::Array(arr)))
    } else {
        Ok(None)
    }
}

fn try_object(obj: &Bound<PyAny>) -> PyResult<Option<JsonValue>> {
    if let Ok(dict) = obj.cast::<PyDict>() {
        let mut map = HashMap::with_capacity(dict.len());
        for (key, val) in dict.iter() {
            let key_str = key
                .extract::<String>()
                .map_err(|_| PyValueError::new_err("JSON object keys must be strings"))?;
            map.insert(key_str, py_to_json_value(&val)?);
        }
        Ok(Some(JsonValue::Object(map)))
    } else {
        Ok(None)
    }
}
