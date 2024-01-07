use pyo3::prelude::*;
extern crate ndarray;

mod aedat;
mod dat;
mod event_stream;
mod raw;
mod utilities;

#[pymodule]
fn faery(python: pyo3::prelude::Python, module: &pyo3::prelude::PyModule) -> PyResult<()> {
    {
        let submodule = PyModule::new(python, "aedat")?;
        submodule.add_class::<aedat::Decoder>()?;
        module.add_submodule(submodule)?;
    }
    {
        let submodule = PyModule::new(python, "dat")?;
        submodule.add_class::<aedat::Decoder>()?;
        module.add_submodule(submodule)?;
    }
    {
        let submodule = PyModule::new(python, "event_stream")?;
        submodule.add_class::<aedat::Decoder>()?;
        module.add_submodule(submodule)?;
    }
    {
        let submodule = PyModule::new(python, "raw")?;
        submodule.add_class::<aedat::Decoder>()?;
        module.add_submodule(submodule)?;
    }
    Ok(())
}
