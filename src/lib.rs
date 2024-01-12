use pyo3::prelude::*;
extern crate ndarray;

mod aedat;
mod dat;
mod event_stream;
mod evt;
mod utilities;

#[pymodule]
fn faery(python: Python, module: &PyModule) -> PyResult<()> {
    {
        let submodule = PyModule::new(python, "aedat")?;
        submodule.add_class::<aedat::Decoder>()?;
        module.add_submodule(submodule)?;
    }
    {
        let submodule = PyModule::new(python, "dat")?;
        submodule.add_class::<dat::Decoder>()?;
        module.add_submodule(submodule)?;
    }
    {
        let submodule = PyModule::new(python, "event_stream")?;
        submodule.add_class::<event_stream::Decoder>()?;
        module.add_submodule(submodule)?;
    }
    {
        let submodule = PyModule::new(python, "evt")?;
        submodule.add_class::<evt::Decoder>()?;
        module.add_submodule(submodule)?;
    }
    Ok(())
}
