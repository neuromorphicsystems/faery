mod decoder;

use crate::utilities;

use numpy::Element;
use pyo3::prelude::*;

impl From<decoder::Error> for pyo3::PyErr {
    fn from(error: decoder::Error) -> Self {
        pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(error.to_string())
    }
}

impl From<decoder::PacketError> for pyo3::PyErr {
    fn from(error: decoder::PacketError) -> Self {
        pyo3::PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(error.to_string())
    }
}

#[pyclass]
pub struct Decoder {
    decoder: decoder::Decoder,
}

#[pymethods]
impl Decoder {
    #[new]
    fn new(path: &pyo3::types::PyAny) -> Result<Self, pyo3::PyErr> {
        pyo3::Python::with_gil(|python| -> Result<Self, pyo3::PyErr> {
            match utilities::python_path_to_string(python, path) {
                Ok(result) => match decoder::Decoder::new(result) {
                    Ok(result) => Ok(Decoder { decoder: result }),
                    Err(error) => Err(pyo3::PyErr::from(error)),
                },
                Err(error) => Err(error),
            }
        })
    }

    #[getter]
    fn event_stream_type(&self) -> PyResult<String> {
        Ok(match self.decoder.event_stream_type {
            decoder::Type::Generic => "generic",
            decoder::Type::Dvs => "dvs",
            decoder::Type::Atis => "atis",
            decoder::Type::Color => "color",
        }
        .to_owned())
    }

    #[getter]
    fn width(&self) -> PyResult<Option<u16>> {
        Ok(self.decoder.width)
    }

    #[getter]
    fn height(&self) -> PyResult<Option<u16>> {
        Ok(self.decoder.height)
    }

    fn __iter__(shell: pyo3::PyRefMut<Self>) -> PyResult<pyo3::prelude::Py<Decoder>> {
        Ok(shell.into())
    }

    fn __next__(mut shell: pyo3::PyRefMut<Self>) -> PyResult<Option<PyObject>> {
        let packet = match shell.decoder.next() {
            Ok(result) => match result {
                Some(result) => result,
                None => return Ok(None),
            },
            Err(result) => return Err(result.into()),
        };
        pyo3::Python::with_gil(|python| -> PyResult<Option<PyObject>> {
            let python_packet = pyo3::types::PyDict::new(python);
            match packet {
                decoder::Packet::Generic(events) => {
                    let mut length = events.len() as numpy::npyffi::npy_intp;
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
                            "bytes",
                            numpy::PyArrayDescr::object(python).num(),
                        );
                        let mut dtype: *mut numpy::npyffi::PyArray_Descr = std::ptr::null_mut();
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
                            ) as *mut u8;
                            let event = &events[index as usize];
                            *(event_cell.offset(0) as *mut u64) = event.t;
                            *(event_cell.offset(8) as *mut *mut pyo3::ffi::PyObject) =
                                pyo3::ffi::PyBytes_FromStringAndSize(
                                    event.bytes.as_ptr() as *const i8,
                                    event.bytes.len() as pyo3::ffi::Py_ssize_t,
                                );
                        }
                        PyObject::from_owned_ptr(python, array)
                    })?;
                }
                decoder::Packet::Dvs(events) => {
                    let mut length = events.len() as numpy::npyffi::npy_intp;
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
                            ) as *mut u8;
                            let event = events[index as usize];
                            *(event_cell.offset(0) as *mut u64) = event.t;
                            *(event_cell.offset(8) as *mut u16) = event.x;
                            *(event_cell.offset(10) as *mut u16) = event.y;
                            *(event_cell.offset(12) as *mut u8) = event.polarity as u8;
                        }
                        PyObject::from_owned_ptr(python, array)
                    })?;
                }
                decoder::Packet::Atis(events) => {
                    let mut length = events.len() as numpy::npyffi::npy_intp;
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
                        let mut dtype: *mut numpy::npyffi::PyArray_Descr = std::ptr::null_mut();
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
                            ) as *mut u8;
                            let event = events[index as usize];
                            *(event_cell.offset(0) as *mut u64) = event.t;
                            *(event_cell.offset(8) as *mut u16) = event.x;
                            *(event_cell.offset(10) as *mut u16) = event.y;
                            match event.polarity {
                                neuromorphic_types::AtisPolarity::Off => {
                                    *(event_cell.offset(12) as *mut u8) = 0;
                                    *(event_cell.offset(13) as *mut u8) = 0;
                                }
                                neuromorphic_types::AtisPolarity::On => {
                                    *(event_cell.offset(12) as *mut u8) = 0;
                                    *(event_cell.offset(13) as *mut u8) = 1;
                                }
                                neuromorphic_types::AtisPolarity::ExposureStart => {
                                    *(event_cell.offset(12) as *mut u8) = 1;
                                    *(event_cell.offset(13) as *mut u8) = 0;
                                }
                                neuromorphic_types::AtisPolarity::ExposureEnd => {
                                    *(event_cell.offset(12) as *mut u8) = 1;
                                    *(event_cell.offset(13) as *mut u8) = 1;
                                }
                            }
                        }
                        PyObject::from_owned_ptr(python, array)
                    })?;
                }
                decoder::Packet::Color(events) => {
                    let mut length = events.len() as numpy::npyffi::npy_intp;
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
                        let mut dtype: *mut numpy::npyffi::PyArray_Descr = std::ptr::null_mut();
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
                            ) as *mut u8;
                            let event = events[index as usize];
                            *(event_cell.offset(0) as *mut u64) = event.t;
                            *(event_cell.offset(8) as *mut u16) = event.x;
                            *(event_cell.offset(10) as *mut u16) = event.y;
                            *(event_cell.offset(12) as *mut u8) = event.r;
                            *(event_cell.offset(13) as *mut u8) = event.g;
                            *(event_cell.offset(14) as *mut u8) = event.b;
                        }
                        PyObject::from_owned_ptr(python, array)
                    })?;
                }
            }
            Ok(Some(python_packet.into()))
        })
    }
}
