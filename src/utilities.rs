use pyo3::prelude::*;

pub unsafe fn set_dtype_as_list_field(
    python: pyo3::Python,
    list: *mut pyo3::ffi::PyObject,
    index: i32,
    name: &str,
    numpy_type: core::ffi::c_int,
) {
    let tuple = pyo3::ffi::PyTuple_New(2);
    if pyo3::ffi::PyTuple_SetItem(
        tuple,
        0 as pyo3::ffi::Py_ssize_t,
        pyo3::ffi::PyUnicode_FromStringAndSize(
            name.as_ptr() as *const core::ffi::c_char,
            name.len() as pyo3::ffi::Py_ssize_t,
        ),
    ) < 0
    {
        panic!("PyTuple_SetItem 0 failed");
    }
    if pyo3::ffi::PyTuple_SetItem(
        tuple,
        1 as pyo3::ffi::Py_ssize_t,
        numpy::PY_ARRAY_API.PyArray_TypeObjectFromType(python, numpy_type),
    ) < 0
    {
        panic!("PyTuple_SetItem 1 failed");
    }
    if pyo3::ffi::PyList_SetItem(list, index as pyo3::ffi::Py_ssize_t, tuple) < 0 {
        panic!("PyList_SetItem failed");
    }
}

pub fn python_path_to_string(
    python: pyo3::prelude::Python,
    path: &pyo3::types::PyAny,
) -> PyResult<String> {
    if let Ok(result) = path.downcast::<pyo3::types::PyString>() {
        return Ok(result.to_string());
    }
    if let Ok(result) = path.downcast::<pyo3::types::PyBytes>() {
        return Ok(result.to_string());
    }
    let fspath_result = path.to_object(python).call_method0(python, "__fspath__")?;
    {
        let fspath_as_string: PyResult<&pyo3::types::PyString> = fspath_result.extract(python);
        if let Ok(result) = fspath_as_string {
            return Ok(result.to_string());
        }
    }
    let fspath_as_bytes: &pyo3::types::PyBytes = fspath_result.extract(python)?;
    Ok(fspath_as_bytes.to_string())
}
