use std::borrow::Cow;
use std::convert::TryInto;

use serde::{Deserialize, Serialize};

#[cfg(feature = "python-bindings")]
use pyo3::prelude::*;

#[cfg(feature = "python-bindings")]
use pyo3::exceptions as pyexceptions;

#[cfg(feature = "python-bindings")]
use pyo3::{FromPyObject, IntoPy, PyErr, PyResult};

#[cfg(feature = "python-bindings")]
use pyo3::types::*;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Value {
    String(String),
    F64(f64),
    I64(i64),
    U64(u64),
    Bool(bool),
    #[cfg(feature = "json")]
    Json(Box<serde_json::Value>),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum ValueType {
    String,
    F64,
    I64,
    U64,
    Bool,
    #[cfg(feature = "json")]
    Json,
}

///Conversions
impl From<&i32> for Value {
    fn from(i: &i32) -> Self {
        Self::I64(*i as i64)
    }
}

impl From<&u32> for Value {
    fn from(i: &u32) -> Self {
        Self::U64(*i as u64)
    }
}

impl From<&i64> for Value {
    fn from(i: &i64) -> Self {
        Self::I64(*i)
    }
}

impl From<&u64> for Value {
    fn from(i: &u64) -> Self {
        Self::U64(*i)
    }
}

impl From<i32> for Value {
    fn from(i: i32) -> Self {
        Self::I64(i as i64)
    }
}

impl From<u32> for Value {
    fn from(i: u32) -> Self {
        Self::U64(i as u64)
    }
}

impl From<i64> for Value {
    fn from(i: i64) -> Self {
        Self::I64(i)
    }
}

impl From<u64> for Value {
    fn from(i: u64) -> Self {
        Self::U64(i)
    }
}

impl From<&bool> for Value {
    fn from(b: &bool) -> Self {
        Self::Bool(*b)
    }
}

impl From<bool> for Value {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}

impl From<f64> for Value {
    fn from(f: f64) -> Self {
        Self::F64(f)
    }
}

impl From<&str> for Value {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<String> for Value {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

#[cfg(feature = "json")]
impl From<serde_json::Value> for Value {
    fn from(json: serde_json::Value) -> Self {
        Self::Json(Box::new(json))
    }
}

#[cfg(feature = "json")]
impl From<Box<serde_json::Value>> for Value {
    fn from(json: Box<serde_json::Value>) -> Self {
        Self::Json(json)
    }
}

impl TryInto<i64> for Value {
    type Error = ();

    fn try_into(self) -> Result<i64, ()> {
        if let Self::I64(i) = self {
            Ok(i)
        } else {
            Err(())
        }
    }
}

impl TryInto<u64> for Value {
    type Error = ();

    fn try_into(self) -> Result<u64, ()> {
        if let Self::U64(u) = self {
            Ok(u)
        } else {
            Err(())
        }
    }
}

impl TryInto<bool> for Value {
    type Error = ();

    fn try_into(self) -> Result<bool, ()> {
        if let Self::Bool(b) = self {
            Ok(b)
        } else {
            Err(())
        }
    }
}

impl TryInto<f64> for Value {
    type Error = ();

    fn try_into(self) -> Result<f64, ()> {
        if let Self::F64(f) = self {
            Ok(f)
        } else {
            Err(())
        }
    }
}

impl TryInto<String> for Value {
    type Error = ();

    fn try_into(self) -> Result<String, ()> {
        if let Self::String(s) = self {
            Ok(s)
        } else {
            Err(())
        }
    }
}

impl Value {
    pub fn serialize_inner(&self) -> Vec<u8> {
        #[cfg(feature = "json")]
        if let Self::Json(v) = self {
            return serde_json::to_vec(v).unwrap();
        }

        match &self {
            Self::String(v) => bincode::serialize(v),
            Self::F64(v) => bincode::serialize(v),
            Self::I64(v) => bincode::serialize(v),
            Self::U64(v) => bincode::serialize(v),
            Self::Bool(v) => bincode::serialize(v),
            #[cfg(feature = "json")]
            Self::Json(_) => panic!("invalid state"),
        }
        .expect("Failed to serialize inner value")
    }

    pub fn from_bytes(data: &[u8], value_type: &ValueType) -> Result<Value, bincode::Error> {
        let val = match value_type {
            ValueType::String => {
                let v = bincode::deserialize(data)?;
                Value::String(v)
            }
            ValueType::F64 => {
                let v = bincode::deserialize(data)?;
                Value::F64(v)
            }
            ValueType::I64 => {
                let v = bincode::deserialize(data)?;
                Value::I64(v)
            }
            ValueType::U64 => {
                let v = bincode::deserialize(data)?;
                Value::U64(v)
            }
            ValueType::Bool => {
                let v = bincode::deserialize(data)?;
                Value::Bool(v)
            }
            #[cfg(feature = "json")]
            ValueType::Json => {
                let v = serde_json::from_slice(data).unwrap();
                Value::Json(Box::new(v))
            }
        };

        Ok(val)
    }
}

#[cfg(feature = "python-bindings")]
impl FromPyObject<'_> for ValueType {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        let typename: Cow<'_, str> = if let Ok(pytype) = PyAny::downcast::<PyType>(obj) {
            pytype.name().expect("Failed to get typename")
        } else if let Ok(string) = PyAny::downcast::<PyString>(obj) {
            Cow::Borrowed(string.to_str().unwrap())
        } else {
            return Err(PyErr::new::<pyexceptions::PyTypeError, _>(
                "Failed to convert PyObject to ValueType. Need string or python type.",
            ));
        };

        #[cfg(feature = "json")]
        if typename == "json" {
            return Ok(ValueType::Json);
        }

        if typename == "int" || typename == "i64" {
            Ok(ValueType::I64)
        } else if typename == "u64" {
            Ok(ValueType::U64)
        } else if typename == "str" {
            Ok(ValueType::String)
        } else if typename == "bool" {
            Ok(ValueType::Bool)
        } else {
            Err(PyErr::new::<pyexceptions::PyTypeError, _>(format!(
                "Cannot convert to ValueType. Got '{}'.",
                typename
            )))
        }
    }
}

#[cfg(feature = "python-bindings")]
impl FromPyObject<'_> for Value {
    fn extract(obj: &PyAny) -> PyResult<Self> {
        if let Ok(string) = PyAny::downcast::<PyString>(obj) {
            let rs_str: String = string.extract()?;
            Ok(rs_str.into())
        } else if let Ok(pyfloat) = PyAny::downcast::<PyFloat>(obj) {
            let f: f64 = pyfloat.extract()?;
            Ok(f.into())
        } else if let Ok(pyint) = PyAny::downcast::<PyLong>(obj) {
            let i: i64 = pyint.extract()?;
            Ok(i.into())
        } else if let Ok(pyint) = PyAny::downcast::<PyInt>(obj) {
            let i: i64 = pyint.extract()?;
            Ok(i.into())
        } else {
            Err(PyErr::new::<pyexceptions::PyTypeError, _>(
                "Failed to convert PyObject to Value",
            ))
        }
    }
}

#[cfg(feature = "python-bindings")]
impl IntoPy<PyObject> for Value {
    fn into_py(self, py: Python) -> PyObject {
        match self {
            Value::String(string) => string.into_py(py),
            Value::Bool(b) => b.into_py(py),
            Value::I64(integer) => integer.into_py(py),
            Value::F64(f) => f.into_py(py),
            Value::U64(u) => u.into_py(py),
            #[cfg(feature = "json")]
            Value::Json(v) => json_to_python(py, *v),
        }
    }
}

#[cfg(all(feature = "json", feature = "python-bindings"))]
fn json_to_python(py: Python, json_value: serde_json::Value) -> PyObject {
    match json_value {
        serde_json::Value::Object(mut dict) => {
            let py_dict = PyDict::new_bound(py);

            for (name, val) in dict.iter_mut() {
                let py_val = json_to_python(py, val.take());
                py_dict.set_item(name.into_py(py), py_val).unwrap();
            }

            py_dict.into_py(py)
        }
        serde_json::Value::Array(mut arr) => {
            let mut items = Vec::new();

            for val in arr.drain(..) {
                items.push(json_to_python(py, val));
            }

            let py_arr = PyList::new_bound(py, items);
            py_arr.into_py(py)
        }
        serde_json::Value::Null => py.None(),
        serde_json::Value::Bool(b) => b.into_py(py),
        serde_json::Value::Number(n) => {
            if n.is_i64() {
                n.as_i64().unwrap().into_py(py)
            } else {
                n.as_u64().unwrap().into_py(py)
            }
        }
        serde_json::Value::String(s) => s.into_py(py),
    }
}

#[cfg(all(feature = "json", feature = "python-bindings"))]
pub fn python_to_json_value(py: Python, obj: Bound<'_, PyAny>) -> PyResult<Value> {
    let j = python_to_json(py, obj)?;
    Ok(Value::Json(Box::new(j)))
}

#[cfg(all(feature = "json", feature = "python-bindings"))]
pub fn python_to_json(py: Python, obj: Bound<'_, PyAny>) -> PyResult<serde_json::Value> {
    if obj.is_none() {
        Ok(serde_json::Value::Null)
    } else if let Ok(string) = Bound::downcast::<PyString>(&obj) {
        let rs_str: String = string.extract()?;
        Ok(rs_str.into())
    } else if let Ok(pyfloat) = Bound::downcast::<PyFloat>(&obj) {
        let f: f64 = pyfloat.extract()?;
        Ok(f.into())
    } else if let Ok(pyint) = Bound::downcast::<PyLong>(&obj) {
        // Pyint might not fit into signed integer so switch to u64 if needed
        match pyint.extract::<i64>() {
            Ok(i) => Ok(i.into()),
            Err(err) => {
                if err.is_instance_of::<pyexceptions::PyOverflowError>(py) {
                    let u: u64 = pyint.extract()?;
                    Ok(u.into())
                } else {
                    Err(err)
                }
            }
        }
    } else if let Ok(pyint) = Bound::downcast::<PyInt>(&obj) {
        let i: i64 = pyint.extract()?;
        Ok(i.into())
    } else if let Ok(pyarr) = Bound::downcast::<PyList>(&obj) {
        let mut result = Vec::new();

        for elem in pyarr.iter() {
            result.push(python_to_json(py, elem)?);
        }

        Ok(result.into())
    } else if let Ok(pydict) = Bound::downcast::<PyDict>(&obj) {
        let mut result = serde_json::Map::new();

        for (name, elem) in pydict.iter() {
            let key: String = name.extract()?;
            result.insert(key, python_to_json(py, elem)?);
        }

        Ok(result.into())
    } else {
        Err(PyErr::new::<pyexceptions::PyTypeError, _>(
            "Failed to convert PyObject to JSON Value",
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{json_to_python, python_to_json, Value, ValueType};
    use pyo3::Python;
    use serde_json::json;

    #[test]
    fn serialize_json() {
        let j = json!({ "value": 42 });

        let val = Value::Json(Box::new(j));
        let data = val.serialize_inner();

        let val2 = Value::from_bytes(&data, &ValueType::Json).unwrap();

        assert_eq!(val, val2);
    }

    #[test]
    fn json_to_py() {
        pyo3::prepare_freethreaded_python();

        Python::with_gil(|py| {
            let json_obj =
                json!({ "value1": u64::MAX, "value2": i64::MIN, "list": ["a", "b", "c"] });
            let py_obj = json_to_python(py, json_obj.clone());
            let json_obj2 = python_to_json(py, py_obj.into_bound(py)).unwrap();

            assert_eq!(json_obj, json_obj2);
        });
    }
}
