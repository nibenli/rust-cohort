use crate::{JsonError, JsonParser, JsonValue};
use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::{PyBool, PyDict, PyList};
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

// Module registration
#[pymodule]
fn _rust_json_parser(m: &Bound<PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(parse_json, m)?)?;
    m.add_function(wrap_pyfunction!(parse_json_file, m)?)?;
    m.add_function(wrap_pyfunction!(dumps, m)?)?;
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
