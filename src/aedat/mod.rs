mod decoder;

use crate::utilities;

use ndarray::IntoDimension;
use numpy::convert::ToPyArray;
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

    fn id_to_stream(&self, python: Python) -> PyResult<PyObject> {
        match self.decoder {
            Some(ref decoder) => {
                let python_id_to_stream = pyo3::types::PyDict::new(python);
                for (id, stream) in decoder.id_to_stream.iter() {
                    let python_stream = pyo3::types::PyDict::new(python);
                    match stream.content {
                        decoder::StreamContent::Events => {
                            python_stream.set_item("type", "events")?;
                            python_stream.set_item("width", stream.width)?;
                            python_stream.set_item("height", stream.height)?;
                        }
                        decoder::StreamContent::Frame => {
                            python_stream.set_item("type", "frame")?;
                            python_stream.set_item("width", stream.width)?;
                            python_stream.set_item("height", stream.height)?;
                        }
                        decoder::StreamContent::Imus => python_stream.set_item("type", "imus")?,
                        decoder::StreamContent::Triggers => {
                            python_stream.set_item("type", "triggers")?
                        }
                    }
                    python_id_to_stream.set_item(id, python_stream)?;
                }
                Ok(python_id_to_stream.into())
            }
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
            python_packet.set_item("stream_id", packet.stream_id)?;
            match packet.stream.content {
                decoder::StreamContent::Events => {
                    let events = match decoder::events_generated::size_prefixed_root_as_event_packet(
                        packet.buffer,
                    ) {
                        Ok(result) => match result.elements() {
                            Some(result) => result,
                            None => return Err(decoder::PacketError::EmptyEventsPacket.into()),
                        },
                        Err(_) => return Err(decoder::PacketError::MissingPacketSizePrefix.into()),
                    };
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
                            let event = events.get(index as usize);
                            let x = event.x();
                            let y = event.y();
                            if x < 0 || x >= packet.stream.width as i16 {
                                return Err(decoder::PacketError::XOverflow {
                                    x,
                                    width: packet.stream.width,
                                }
                                .into());
                            }
                            if y < 0 || y >= packet.stream.height as i16 {
                                return Err(decoder::PacketError::YOverflow {
                                    y,
                                    height: packet.stream.height,
                                }
                                .into());
                            }
                            let mut event_array = [0u8; 13];
                            event_array[0..8].copy_from_slice(&(event.t() as u64).to_ne_bytes());
                            event_array[8..10].copy_from_slice(&(event.x() as u16).to_ne_bytes());
                            event_array[10..12].copy_from_slice(&(event.y() as u16).to_ne_bytes());
                            event_array[12] = if event.on() { 1 } else { 0 };
                            std::ptr::copy(event_array.as_ptr(), event_cell, event_array.len());
                        }
                        PyObject::from_owned_ptr(python, array)
                    })?;
                }
                decoder::StreamContent::Frame => {
                    let frame =
                        match decoder::frame_generated::size_prefixed_root_as_frame(packet.buffer)
                        {
                            Ok(result) => result,
                            Err(_) => {
                                return Err(PyErr::from(
                                    decoder::PacketError::MissingPacketSizePrefix,
                                ))
                            }
                        };
                    let python_frame = pyo3::types::PyDict::new(python);
                    python_frame.set_item("t", frame.t())?;
                    python_frame.set_item("begin_t", frame.begin_t())?;
                    python_frame.set_item("end_t", frame.end_t())?;
                    python_frame.set_item("exposure_begin_t", frame.exposure_begin_t())?;
                    python_frame.set_item("exposure_end_t", frame.exposure_end_t())?;
                    python_frame.set_item(
                        "format",
                        match frame.format() {
                            decoder::frame_generated::FrameFormat::Gray => "L",
                            decoder::frame_generated::FrameFormat::Bgr => "RGB",
                            decoder::frame_generated::FrameFormat::Bgra => "RGBA",
                            _ => return Err(PyErr::from(decoder::PacketError::UnknownFrameFormat)),
                        },
                    )?;
                    python_frame.set_item("width", frame.width())?;
                    python_frame.set_item("height", frame.height())?;
                    python_frame.set_item("offset_x", frame.offset_x())?;
                    python_frame.set_item("offset_y", frame.offset_y())?;
                    match frame.format() {
                        decoder::frame_generated::FrameFormat::Gray => {
                            let dimensions =
                                [frame.height() as usize, frame.width() as usize].into_dimension();
                            python_frame.set_item(
                                "pixels",
                                match frame.pixels() {
                                    Some(result) => {
                                        result.bytes().to_pyarray(python).reshape(dimensions)?
                                    }
                                    None => numpy::array::PyArray2::<u8>::zeros(
                                        python, dimensions, false,
                                    ),
                                },
                            )?;
                        }
                        decoder::frame_generated::FrameFormat::Bgr
                        | decoder::frame_generated::FrameFormat::Bgra => {
                            let channels =
                                if frame.format() == decoder::frame_generated::FrameFormat::Bgr {
                                    3_usize
                                } else {
                                    4_usize
                                };
                            let dimensions =
                                [frame.height() as usize, frame.width() as usize, channels]
                                    .into_dimension();
                            python_frame.set_item(
                                "pixels",
                                match frame.pixels() {
                                    Some(result) => {
                                        let mut pixels = result.bytes().to_owned();
                                        for index in 0..(pixels.len() / channels) {
                                            pixels.swap(index * channels, index * channels + 2);
                                        }
                                        pixels.to_pyarray(python).reshape(dimensions)?
                                    }
                                    None => numpy::array::PyArray3::<u8>::zeros(
                                        python, dimensions, false,
                                    ),
                                },
                            )?;
                        }
                        _ => return Err(PyErr::from(decoder::PacketError::UnknownFrameFormat)),
                    }
                    python_packet.set_item("frame", python_frame)?;
                }
                decoder::StreamContent::Imus => {
                    let imus = match decoder::imus_generated::size_prefixed_root_as_imu_packet(
                        packet.buffer,
                    ) {
                        Ok(result) => match result.elements() {
                            Some(result) => result,
                            None => {
                                return Err(PyErr::from(decoder::PacketError::EmptyEventsPacket))
                            }
                        },
                        Err(_) => {
                            return Err(PyErr::from(decoder::PacketError::MissingPacketSizePrefix))
                        }
                    };
                    let mut length = imus.len() as numpy::npyffi::npy_intp;
                    python_packet.set_item("imus", unsafe {
                        let dtype_as_list = pyo3::ffi::PyList_New(11_isize);
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
                            "temperature",
                            f32::get_dtype(python).num(),
                        );
                        utilities::set_dtype_as_list_field(
                            python,
                            dtype_as_list,
                            2,
                            "accelerometer_x",
                            f32::get_dtype(python).num(),
                        );
                        utilities::set_dtype_as_list_field(
                            python,
                            dtype_as_list,
                            3,
                            "accelerometer_y",
                            f32::get_dtype(python).num(),
                        );
                        utilities::set_dtype_as_list_field(
                            python,
                            dtype_as_list,
                            4,
                            "accelerometer_z",
                            f32::get_dtype(python).num(),
                        );
                        utilities::set_dtype_as_list_field(
                            python,
                            dtype_as_list,
                            5,
                            "gyroscope_x",
                            f32::get_dtype(python).num(),
                        );
                        utilities::set_dtype_as_list_field(
                            python,
                            dtype_as_list,
                            6,
                            "gyroscope_y",
                            f32::get_dtype(python).num(),
                        );
                        utilities::set_dtype_as_list_field(
                            python,
                            dtype_as_list,
                            7,
                            "gyroscope_z",
                            f32::get_dtype(python).num(),
                        );
                        utilities::set_dtype_as_list_field(
                            python,
                            dtype_as_list,
                            8,
                            "magnetometer_x",
                            f32::get_dtype(python).num(),
                        );
                        utilities::set_dtype_as_list_field(
                            python,
                            dtype_as_list,
                            9,
                            "magnetometer_y",
                            f32::get_dtype(python).num(),
                        );
                        utilities::set_dtype_as_list_field(
                            python,
                            dtype_as_list,
                            10,
                            "magnetometer_z",
                            f32::get_dtype(python).num(),
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
                        let mut index = 0_isize;
                        for imu in imus {
                            let imu_cell = numpy::PY_ARRAY_API.PyArray_GetPtr(
                                python,
                                array as *mut numpy::npyffi::PyArrayObject,
                                &mut index as *mut numpy::npyffi::npy_intp,
                            ) as *mut u8;
                            let mut imu_array = [0u8; 48];
                            imu_array[0..8].copy_from_slice(&(imu.t() as u64).to_ne_bytes());
                            imu_array[8..12].copy_from_slice(&(imu.temperature()).to_ne_bytes());
                            imu_array[12..16]
                                .copy_from_slice(&(imu.accelerometer_x()).to_ne_bytes());
                            imu_array[16..20]
                                .copy_from_slice(&(imu.accelerometer_y()).to_ne_bytes());
                            imu_array[20..24]
                                .copy_from_slice(&(imu.accelerometer_z()).to_ne_bytes());
                            imu_array[24..28].copy_from_slice(&(imu.gyroscope_x()).to_ne_bytes());
                            imu_array[28..32].copy_from_slice(&(imu.gyroscope_y()).to_ne_bytes());
                            imu_array[32..36].copy_from_slice(&(imu.gyroscope_z()).to_ne_bytes());
                            imu_array[36..40]
                                .copy_from_slice(&(imu.magnetometer_x()).to_ne_bytes());
                            imu_array[40..44]
                                .copy_from_slice(&(imu.magnetometer_y()).to_ne_bytes());
                            imu_array[44..48]
                                .copy_from_slice(&(imu.magnetometer_z()).to_ne_bytes());
                            std::ptr::copy(imu_array.as_ptr(), imu_cell, imu_array.len());
                            index += 1_isize;
                        }
                        PyObject::from_owned_ptr(python, array)
                    })?;
                }
                decoder::StreamContent::Triggers => {
                    let triggers =
                        match decoder::triggers_generated::size_prefixed_root_as_trigger_packet(
                            packet.buffer,
                        ) {
                            Ok(result) => match result.elements() {
                                Some(result) => result,
                                None => {
                                    return Err(PyErr::from(
                                        decoder::PacketError::EmptyEventsPacket,
                                    ))
                                }
                            },
                            Err(_) => {
                                return Err(PyErr::from(
                                    decoder::PacketError::MissingPacketSizePrefix,
                                ))
                            }
                        };
                    let mut length = triggers.len() as numpy::npyffi::npy_intp;
                    python_packet.set_item("triggers", unsafe {
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
                            "source",
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
                        let mut index = 0_isize;
                        for trigger in triggers {
                            let trigger_cell = numpy::PY_ARRAY_API.PyArray_GetPtr(
                                python,
                                array as *mut numpy::npyffi::PyArrayObject,
                                &mut index as *mut numpy::npyffi::npy_intp,
                            ) as *mut u8;
                            let mut trigger_array = [0u8; 9];
                            trigger_array[0..8]
                                .copy_from_slice(&(trigger.t() as u64).to_ne_bytes());
                            use decoder::triggers_generated::TriggerSource;
                            trigger_array[8] = match trigger.source() {
                                TriggerSource::TimestampReset => 0_u8,
                                TriggerSource::ExternalSignalRisingEdge => 1_u8,
                                TriggerSource::ExternalSignalFallingEdge => 2_u8,
                                TriggerSource::ExternalSignalPulse => 3_u8,
                                TriggerSource::ExternalGeneratorRisingEdge => 4_u8,
                                TriggerSource::ExternalGeneratorFallingEdge => 5_u8,
                                TriggerSource::FrameBegin => 6_u8,
                                TriggerSource::FrameEnd => 7_u8,
                                TriggerSource::ExposureBegin => 8_u8,
                                TriggerSource::ExposureEnd => 9_u8,
                                _ => {
                                    return Err(PyErr::from(
                                        decoder::PacketError::UnknownTriggerSource,
                                    ))
                                }
                            };
                            std::ptr::copy(
                                trigger_array.as_ptr(),
                                trigger_cell,
                                trigger_array.len(),
                            );
                            index += 1_isize;
                        }
                        PyObject::from_owned_ptr(python, array)
                    })?;
                }
            }
            Ok(Some(python_packet.into()))
        })
    }
}
