use pyo3::prelude::*;
use std::io::BufRead;

pub const BUFFER_SIZE: usize = 65536;

pub unsafe fn set_dtype_as_list_field(
    python: Python,
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

pub fn python_path_to_string(python: Python, path: &pyo3::types::PyAny) -> PyResult<String> {
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

pub struct Header {
    pub size: Option<(u16, u16)>,
    pub version: Option<String>,
    pub length: u64,
}

pub fn read_header(
    file: &mut std::io::BufReader<std::fs::File>,
    marker: char,
) -> Result<Header, std::io::Error> {
    let mut buffer = String::new();
    let mut width: Option<u16> = None;
    let mut height: Option<u16> = None;
    let mut version: Option<String> = None;
    let mut length = 0;
    loop {
        buffer.clear();
        let bytes_read = match file.read_line(&mut buffer) {
            Ok(bytes_read) => bytes_read,
            Err(error) => match error.kind() {
                std::io::ErrorKind::InvalidData => 0,
                _ => return Err(error),
            },
        };
        if bytes_read == 0 || !buffer.starts_with(marker) {
            break;
        }
        length += bytes_read as u64;
        let words: Vec<&str> = buffer[1..]
            .trim()
            .split(" ")
            .map(|word| word.trim())
            .collect();
        if words.len() > 1 {
            match words[0] {
                "Version" => {
                    version = Some(words[1].to_lowercase());
                }
                "Width" => {
                    if let Ok(width_candidate) = words[1].parse() {
                        width = Some(width_candidate);
                    }
                }
                "Height" => {
                    if let Ok(height_candidate) = words[1].parse() {
                        height = Some(height_candidate);
                    }
                }
                "geometry" => {
                    let subwords: Vec<&str> = words[1].split("x").collect();
                    if subwords.len() == 2 {
                        if let Ok(width_candidate) = subwords[0].parse() {
                            if let Ok(height_candidate) = subwords[1].parse() {
                                width = Some(width_candidate);
                                height = Some(height_candidate);
                            }
                        }
                    }
                }
                "format" => {
                    version = Some(match words[1] {
                        "EVT2" | "evt2" | "EVT2.0" | "evt2.0" => "2".to_owned(),
                        "EVT2.1" | "evt2.1" => "2.1".to_owned(),
                        "EVT3" | "evt3" | "EVT3.0" | "evt3.0" => "3".to_owned(),
                        word => word.to_owned(),
                    });
                }
                "evt" => {
                    version = Some(match words[1] {
                        "2" | "2.0" => "2".to_owned(),
                        "2.1" => "2.1".to_owned(),
                        "3" | "3.0" => "3".to_owned(),
                        word => word.to_owned(),
                    });
                }
                _ => (),
            }
        }
    }
    if let Some(width) = width {
        if let Some(height) = height {
            return Ok(Header {
                size: Some((width, height)),
                version,
                length,
            });
        }
    }
    Ok(Header {
        size: None,
        version,
        length,
    })
}
