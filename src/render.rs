use pyo3::prelude::*;

use crate::types;

struct BufferedArray {
    inner: PyObject,
    array: *mut numpy::npyffi::PyArrayObject,
    length: isize,
    index: isize,
}

unsafe impl Send for BufferedArray {}

struct Inner {
    next_frame_t: u64,
    frame_duration: u64,
    frame_index: u64,
    frame_count: u64,
    decay: Decay,
    buffered_array: Option<BufferedArray>,
    ts_and_polarities: Vec<(u64, neuromorphic_types::DvsPolarity)>,
}

#[pyclass]
pub struct RenderIterator {
    inner: Option<Inner>,
}

#[derive(Debug, Clone, Copy)]
enum Decay {
    Exponential(f64),
    Linear(f64),
    Step(u64),
}

#[pymethods]
impl RenderIterator {
    #[new]
    fn new(
        parent: &pyo3::Bound<'_, pyo3::types::PyAny>,
        dimensions: (u16, u16),
        next_frame_t: u64,
        frame_duration: u64,
        frame_count: u64,
        decay: String,
        tau: u64,
    ) -> Result<Self, PyErr> {
        Python::with_gil(|python| -> Result<Self, PyErr> {
            let decay = match decay.as_str() {
                "exponential" => Decay::Exponential(-1.0 / (tau as f64)),
                "linear" => Decay::Linear(-1.0 / (tau as f64)),
                "step" => Decay::Step(tau),
                decay => {
                    return Err(PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!(
                        "unknown decay \"{decay}\" (expected \"exponential\", \"linear\", or \"step\")"
                    )));
                }
            };
            Ok(RenderIterator {
                inner: Some(Inner {
                    next_frame_t,
                    frame_duration,
                    frame_index: 0,
                    frame_count,
                    decay,
                    buffered_array: None,
                    ts_and_polarities: vec![
                        (0, neuromorphic_types::DvsPolarity::Off);
                        dimensions.0 as usize * dimensions.1 as usize
                    ],
                }),
            })
        })
    }

    fn __iter__(shell: PyRefMut<Self>) -> PyResult<Py<RenderIterator>> {
        Ok(shell.into())
    }

    /*
    fn __next__(mut shell: PyRefMut<Self>) -> PyResult<Option<PyObject>> {
        Python::with_gil(|python| -> PyResult<Option<PyObject>> {
            match shell.inner {
                Some(ref mut decoder) => loop {
                    if let Some(buffered_array) = decoder.buffered_array.take() {
                        while buffered_array.index < buffered_array.length {
                            unsafe {
                                let event_cell: *mut neuromorphic_types::DvsEvent<u64, u16, u16> =
                                    types::array_at(
                                        python,
                                        buffered_array.array,
                                        buffered_array.index,
                                    );
                                if (*event_cell).t >= decoder.next_frame_t {
                                    decoder.buffered_array = Some(buffered_array);

                                    // @DEV render frame
                                    return Ok(Some());
                                }
                            }
                        }
                    }
                },
                None => Err(pyo3::exceptions::PyException::new_err(
                    "__next__ called after __exit__",
                )),
            }
        })
    }
     */

    fn close(&mut self) {
        let _ = self.inner.take();
    }
}

fn render(
    python: Python,
    dimensions: (u16, u16),
    ts_and_polarities: &Vec<(u64, neuromorphic_types::DvsPolarity)>,
    frame_t: u64,
    decay: Decay,
) {
    /*
    unsafe {
        let array = numpy::PyArray2::<f64>::new_bound(
            python,
            (dimensions.0 as usize, dimensions.1 as usize),
            false,
        );
        for y in 0..dimensions.1 as isize {
            for x in 0..dimensions.0 as isize {
                array
            }
        }
    }
     */
}
