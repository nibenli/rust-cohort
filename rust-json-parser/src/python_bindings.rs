use crate::{JsonError, JsonParser, JsonValue};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use pyo3::{Bound, IntoPyObject, PyAny, PyErr};
use std::collections::HashMap;

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

#[pyfunction]
pub fn parse_json<'py>(py: Python<'py>, input: &str) -> PyResult<Bound<'py, PyAny>> {
    let json_value = JsonParser::new(input)?.parse()?;

    let py_object = json_value.into_pyobject(py)?;

    Ok(py_object)
}

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
    if let Ok(b) = obj.extract::<bool>() {
        return Ok(JsonValue::Boolean(b));
    }

    // Check for Number (f64)
    if let Ok(n) = obj.extract::<f64>() {
        return Ok(JsonValue::Number(n));
    }

    // Check for String
    if let Ok(s) = obj.extract::<String>() {
        return Ok(JsonValue::String(s));
    }

    // Check for List (recurse on elements)
    if let Ok(list) = obj.cast::<PyList>() {
        let mut arr = Vec::with_capacity(list.len());
        for item in list.iter() {
            // Recursive call for each element
            arr.push(py_to_json_value(&item)?);
        }
        return Ok(JsonValue::Array(arr));
    }

    // Check for Dictionary (recurse on values)
    if let Ok(dict) = obj.cast::<PyDict>() {
        let mut map = HashMap::with_capacity(dict.len());
        for (key, val) in dict.iter() {
            // Keys must be strings in JSON
            let key_str = key.extract::<String>()?;
            // Recursive call for each value
            map.insert(key_str, py_to_json_value(&val)?);
        }
        return Ok(JsonValue::Object(map));
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
        Some(n) => Ok(format_json_pretty(&json_value, n, 0)),
    }
}

/// Helper function to handle recursive indentation
fn format_json_pretty(val: &JsonValue, indent_size: usize, depth: usize) -> String {
    let current_indent = " ".repeat(depth * indent_size);
    let next_indent = " ".repeat((depth + 1) * indent_size);

    match val {
        JsonValue::Null => "null".to_string(),
        JsonValue::Boolean(b) => b.to_string(),
        JsonValue::Number(n) => n.to_string(),
        JsonValue::String(s) => format!("\"{s}\""),

        JsonValue::Array(arr) => {
            if arr.is_empty() {
                return "[]".to_string();
            }
            let mut parts = Vec::new();
            for item in arr {
                parts.push(format!(
                    "{}{}",
                    next_indent,
                    format_json_pretty(item, indent_size, depth + 1)
                ));
            }
            format!("[\n{}\n{}]", parts.join(",\n"), current_indent)
        }

        JsonValue::Object(obj) => {
            if obj.is_empty() {
                return "{}".to_string();
            }
            let mut parts = Vec::new();
            for (key, value) in obj {
                let formatted_val = format_json_pretty(value, indent_size, depth + 1);
                parts.push(format!("{next_indent}\"{key}\": {formatted_val}"));
            }
            format!("{{\n{}\n{}}}", parts.join(",\n"), current_indent)
        }
    }
}

// Module registration
#[pymodule]
fn _rust_json_parser(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_json, m)?)?;
    m.add_function(wrap_pyfunction!(parse_json_file, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
    m.add("__version__", "0.1.0")?;
    Ok(())
}
