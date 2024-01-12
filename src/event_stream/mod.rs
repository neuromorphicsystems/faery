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
    fn new(path: &pyo3::types::PyAny) -> Result<Self, PyErr> {
        Python::with_gil(|python| -> Result<Self, PyErr> {
            match utilities::python_path_to_string(python, path) {
                Ok(result) => match decoder::Decoder::new(result) {
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
    fn event_type(&self) -> PyResult<String> {
        match self.decoder {
            Some(ref decoder) => Ok(match decoder.event_type {
                decoder::Type::Generic => "generic",
                decoder::Type::Dvs => "dvs",
                decoder::Type::Atis => "atis",
                decoder::Type::Color => "color",
            }
            .to_owned()),
            None => Err(pyo3::exceptions::PyException::new_err(
                "used decoder after __exit__",
            )),
        }
    }

    #[getter]
    fn width(&self) -> PyResult<Option<u16>> {
        match self.decoder {
            Some(ref decoder) => Ok(decoder.width),
            None => Err(pyo3::exceptions::PyException::new_err(
                "used decoder after __exit__",
            )),
        }
    }

    #[getter]
    fn height(&self) -> PyResult<Option<u16>> {
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
            Ok(
                Some(
                    match packet {
                        decoder::Packet::Generic(events) => {
                            let mut length = events.len() as numpy::npyffi::npy_intp;
                            unsafe {
                                let dtype_as_list = pyo3::ffi::PyList_New(2_isize);
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
                                    "bytes",
                                    numpy::PyArrayDescr::object(python).num(),
                                );
                                let mut dtype: *mut numpy::npyffi::PyArray_Descr =
                                    std::ptr::null_mut();
                                if numpy::PY_ARRAY_API.PyArray_DescrConverter(
                                    python,
                                    dtype_as_list,
                                    &mut dtype,
                                ) < 0
                                {
                                    panic!("PyArray_DescrConverter failed");
                                }
                                let array = numpy::PY_ARRAY_API.PyArray_NewFromDescr(
                                    python,
                                    numpy::PY_ARRAY_API.get_type_object(
                                        python,
                                        numpy::npyffi::array::NpyTypes::PyArray_Type,
                                    ),
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
                                    )
                                        as *mut u8;
                                    let event = &events[index as usize];
                                    let mut event_array = [0u8; 8 + std::mem::size_of::<usize>()];
                                    event_array[0..8].copy_from_slice(&event.t.to_ne_bytes());
                                    let pybytes = pyo3::ffi::PyBytes_FromStringAndSize(
                                        event.bytes.as_ptr() as *const i8,
                                        event.bytes.len() as pyo3::ffi::Py_ssize_t,
                                    );
                                    event_array[8..8 + std::mem::size_of::<usize>()]
                                        .copy_from_slice(&(pybytes as usize).to_ne_bytes());
                                    std::ptr::copy(
                                        event_array.as_ptr(),
                                        event_cell,
                                        event_array.len(),
                                    );
                                }
                                PyObject::from_owned_ptr(python, array)
                            }
                        }
                        decoder::Packet::Dvs(events) => {
                            let mut length = events.len() as numpy::npyffi::npy_intp;
                            unsafe {
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
                                let mut dtype: *mut numpy::npyffi::PyArray_Descr =
                                    std::ptr::null_mut();
                                if numpy::PY_ARRAY_API.PyArray_DescrConverter(
                                    python,
                                    dtype_as_list,
                                    &mut dtype,
                                ) < 0
                                {
                                    panic!("PyArray_DescrConverter failed");
                                }
                                let array = numpy::PY_ARRAY_API.PyArray_NewFromDescr(
                                    python,
                                    numpy::PY_ARRAY_API.get_type_object(
                                        python,
                                        numpy::npyffi::array::NpyTypes::PyArray_Type,
                                    ),
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
                                    )
                                        as *mut u8;
                                    std::ptr::copy(
                                        &events[index as usize]
                                            as *const neuromorphic_types::DvsEvent<u64, u16, u16>
                                            as *const u8,
                                        event_cell,
                                        std::mem::size_of::<
                                            neuromorphic_types::DvsEvent<u64, u16, u16>,
                                        >(),
                                    );
                                }
                                PyObject::from_owned_ptr(python, array)
                            }
                        }
                        decoder::Packet::Atis(events) => {
                            let mut length = events.len() as numpy::npyffi::npy_intp;
                            unsafe {
                                let dtype_as_list = pyo3::ffi::PyList_New(5_isize);
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
                                    "exposure",
                                    bool::get_dtype(python).num(),
                                );
                                utilities::set_dtype_as_list_field(
                                    python,
                                    dtype_as_list,
                                    4,
                                    "polarity",
                                    bool::get_dtype(python).num(),
                                );
                                let mut dtype: *mut numpy::npyffi::PyArray_Descr =
                                    std::ptr::null_mut();
                                if numpy::PY_ARRAY_API.PyArray_DescrConverter(
                                    python,
                                    dtype_as_list,
                                    &mut dtype,
                                ) < 0
                                {
                                    panic!("PyArray_DescrConverter failed");
                                }
                                let array = numpy::PY_ARRAY_API.PyArray_NewFromDescr(
                                    python,
                                    numpy::PY_ARRAY_API.get_type_object(
                                        python,
                                        numpy::npyffi::array::NpyTypes::PyArray_Type,
                                    ),
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
                                    )
                                        as *mut u8;
                                    let event = events[index as usize];
                                    let mut event_array = [0u8; 14];
                                    event_array[0..8].copy_from_slice(&event.t.to_ne_bytes());
                                    event_array[8..10].copy_from_slice(&event.x.to_ne_bytes());
                                    event_array[10..12].copy_from_slice(&event.y.to_ne_bytes());
                                    match event.polarity {
                                        neuromorphic_types::AtisPolarity::Off => {
                                            event_array[12] = 0;
                                            event_array[13] = 0;
                                        }
                                        neuromorphic_types::AtisPolarity::On => {
                                            event_array[12] = 0;
                                            event_array[13] = 1;
                                        }
                                        neuromorphic_types::AtisPolarity::ExposureStart => {
                                            event_array[12] = 1;
                                            event_array[13] = 0;
                                        }
                                        neuromorphic_types::AtisPolarity::ExposureEnd => {
                                            event_array[12] = 1;
                                            event_array[13] = 1;
                                        }
                                    }
                                    std::ptr::copy(
                                        event_array.as_ptr(),
                                        event_cell,
                                        event_array.len(),
                                    );
                                }
                                PyObject::from_owned_ptr(python, array)
                            }
                        }
                        decoder::Packet::Color(events) => {
                            let mut length = events.len() as numpy::npyffi::npy_intp;
                            unsafe {
                                let dtype_as_list = pyo3::ffi::PyList_New(6_isize);
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
                                    "r",
                                    u8::get_dtype(python).num(),
                                );
                                utilities::set_dtype_as_list_field(
                                    python,
                                    dtype_as_list,
                                    4,
                                    "g",
                                    u8::get_dtype(python).num(),
                                );
                                utilities::set_dtype_as_list_field(
                                    python,
                                    dtype_as_list,
                                    5,
                                    "b",
                                    u8::get_dtype(python).num(),
                                );
                                let mut dtype: *mut numpy::npyffi::PyArray_Descr =
                                    std::ptr::null_mut();
                                if numpy::PY_ARRAY_API.PyArray_DescrConverter(
                                    python,
                                    dtype_as_list,
                                    &mut dtype,
                                ) < 0
                                {
                                    panic!("PyArray_DescrConverter failed");
                                }
                                let array = numpy::PY_ARRAY_API.PyArray_NewFromDescr(
                                    python,
                                    numpy::PY_ARRAY_API.get_type_object(
                                        python,
                                        numpy::npyffi::array::NpyTypes::PyArray_Type,
                                    ),
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
                                    )
                                        as *mut u8;
                                    std::ptr::copy(
                                        &events[index as usize] as *const decoder::ColorEvent
                                            as *const u8,
                                        event_cell,
                                        std::mem::size_of::<decoder::ColorEvent>(),
                                    );
                                }
                                PyObject::from_owned_ptr(python, array)
                            }
                        }
                    }
                    .into(),
                ),
            )
        })
    }
}
