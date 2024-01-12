mod decoder;

use crate::utilities;

use numpy::Element;
use pyo3::prelude::*;

impl From<decoder::Error> for PyErr {
    fn from(error: decoder::Error) -> Self {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(error.to_string())
    }
}

impl From<decoder::PacketError> for PyErr {
    fn from(error: decoder::PacketError) -> Self {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(error.to_string())
    }
}

#[pyclass]
pub struct Decoder {
    decoder: Option<decoder::Decoder>,
}

#[pymethods]
impl Decoder {
    #[new]
    fn new(
        path: &pyo3::types::PyAny,
        size_fallback: Option<(u16, u16)>,
        version_fallback: Option<String>,
    ) -> Result<Self, PyErr> {
        Python::with_gil(|python| -> Result<Self, PyErr> {
            match utilities::python_path_to_string(python, path) {
                Ok(result) => match decoder::Decoder::new(
                    result,
                    size_fallback,
                    version_fallback
                        .map(|version| decoder::Version::from_string(&version))
                        .transpose()?,
                ) {
                    Ok(result) => Ok(Decoder {
                        decoder: Some(result),
                    }),
                    Err(error) => Err(PyErr::from(error)),
                },
                Err(error) => Err(error),
            }
        })
    }

    #[getter]
    fn width(&self) -> PyResult<u16> {
        match self.decoder {
            Some(ref decoder) => Ok(decoder.width),
            None => Err(pyo3::exceptions::PyException::new_err(
                "used decoder after __exit__",
            )),
        }
    }

    #[getter]
    fn height(&self) -> PyResult<u16> {
        match self.decoder {
            Some(ref decoder) => Ok(decoder.height),
            None => Err(pyo3::exceptions::PyException::new_err(
                "used decoder after __exit__",
            )),
        }
    }

    fn __enter__(slf: Py<Self>) -> Py<Self> {
        slf
    }

    fn __exit__(
        &mut self,
        _exception_type: Option<PyObject>,
        _value: Option<PyObject>,
        _traceback: Option<PyObject>,
    ) -> PyResult<bool> {
        if self.decoder.is_none() {
            return Err(pyo3::exceptions::PyException::new_err(
                "multiple calls to __exit__",
            ));
        }
        let _ = self.decoder.take();
        Ok(false)
    }

    fn __iter__(shell: PyRefMut<Self>) -> PyResult<Py<Decoder>> {
        Ok(shell.into())
    }

    fn __next__(mut shell: PyRefMut<Self>) -> PyResult<Option<PyObject>> {
        let packet = match shell.decoder {
            Some(ref mut decoder) => match decoder.next() {
                Ok(result) => match result {
                    Some(result) => result,
                    None => return Ok(None),
                },
                Err(result) => return Err(result.into()),
            },
            None => {
                return Err(pyo3::exceptions::PyException::new_err(
                    "used decoder after __exit__",
                ))
            }
        };
        Python::with_gil(|python| -> PyResult<Option<PyObject>> {
            let python_packet = pyo3::types::PyDict::new(python);
            if !packet.0.is_empty() {
                let mut length = packet.0.len() as numpy::npyffi::npy_intp;
                python_packet.set_item("events", unsafe {
                    let dtype_as_list = pyo3::ffi::PyList_New(4_isize);
                    utilities::set_dtype_as_list_field(
                        python,
                        dtype_as_list,
                        0,
                        "t",
                        u64::get_dtype(python).num(),
                    );
                    utilities::set_dtype_as_list_field(
                        python,
                        dtype_as_list,
                        1,
                        "x",
                        u16::get_dtype(python).num(),
                    );
                    utilities::set_dtype_as_list_field(
                        python,
                        dtype_as_list,
                        2,
                        "y",
                        u16::get_dtype(python).num(),
                    );
                    utilities::set_dtype_as_list_field(
                        python,
                        dtype_as_list,
                        3,
                        "on",
                        bool::get_dtype(python).num(),
                    );
                    let mut dtype: *mut numpy::npyffi::PyArray_Descr = std::ptr::null_mut();
                    if numpy::PY_ARRAY_API.PyArray_DescrConverter(python, dtype_as_list, &mut dtype)
                        < 0
                    {
                        panic!("PyArray_DescrConverter failed");
                    }
                    let array = numpy::PY_ARRAY_API.PyArray_NewFromDescr(
                        python,
                        numpy::PY_ARRAY_API
                            .get_type_object(python, numpy::npyffi::array::NpyTypes::PyArray_Type),
                        dtype,
                        1_i32,
                        &mut length as *mut numpy::npyffi::npy_intp,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        0_i32,
                        std::ptr::null_mut(),
                    );
                    for mut index in 0_isize..length {
                        let event_cell = numpy::PY_ARRAY_API.PyArray_GetPtr(
                            python,
                            array as *mut numpy::npyffi::PyArrayObject,
                            &mut index as *mut numpy::npyffi::npy_intp,
                        ) as *mut u8;
                        std::ptr::copy(
                            &packet.0[index as usize]
                                as *const neuromorphic_types::DvsEvent<u64, u16, u16>
                                as *const u8,
                            event_cell,
                            std::mem::size_of::<neuromorphic_types::DvsEvent<u64, u16, u16>>(),
                        );
                    }
                    PyObject::from_owned_ptr(python, array)
                })?;
            }
            if !packet.1.is_empty() {
                let mut length = packet.1.len() as numpy::npyffi::npy_intp;
                python_packet.set_item("triggers", unsafe {
                    let dtype_as_list = pyo3::ffi::PyList_New(3_isize);
                    utilities::set_dtype_as_list_field(
                        python,
                        dtype_as_list,
                        0,
                        "t",
                        u64::get_dtype(python).num(),
                    );
                    utilities::set_dtype_as_list_field(
                        python,
                        dtype_as_list,
                        1,
                        "id",
                        u8::get_dtype(python).num(),
                    );
                    utilities::set_dtype_as_list_field(
                        python,
                        dtype_as_list,
                        2,
                        "rising",
                        bool::get_dtype(python).num(),
                    );
                    let mut dtype: *mut numpy::npyffi::PyArray_Descr = std::ptr::null_mut();
                    if numpy::PY_ARRAY_API.PyArray_DescrConverter(python, dtype_as_list, &mut dtype)
                        < 0
                    {
                        panic!("PyArray_DescrConverter failed");
                    }
                    let array = numpy::PY_ARRAY_API.PyArray_NewFromDescr(
                        python,
                        numpy::PY_ARRAY_API
                            .get_type_object(python, numpy::npyffi::array::NpyTypes::PyArray_Type),
                        dtype,
                        1_i32,
                        &mut length as *mut numpy::npyffi::npy_intp,
                        std::ptr::null_mut(),
                        std::ptr::null_mut(),
                        0_i32,
                        std::ptr::null_mut(),
                    );
                    for mut index in 0_isize..length {
                        let trigger_cell = numpy::PY_ARRAY_API.PyArray_GetPtr(
                            python,
                            array as *mut numpy::npyffi::PyArrayObject,
                            &mut index as *mut numpy::npyffi::npy_intp,
                        ) as *mut u8;
                        let trigger = packet.1[index as usize];
                        let mut trigger_array = [0u8; 10];
                        trigger_array[0..8].copy_from_slice(&trigger.t.to_ne_bytes());
                        trigger_array[8] = trigger.id;
                        trigger_array[9] = match trigger.polarity {
                            neuromorphic_types::TriggerPolarity::Falling => 0,
                            neuromorphic_types::TriggerPolarity::Rising => 1,
                        };
                        std::ptr::copy(trigger_array.as_ptr(), trigger_cell, trigger_array.len());
                    }
                    PyObject::from_owned_ptr(python, array)
                })?;
            }
            Ok(Some(python_packet.into()))
        })
    }
}
