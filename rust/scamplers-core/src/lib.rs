pub mod institution;
pub mod person;
pub mod lab;
pub mod chromium;
mod dataset_metadata;
mod sample_metadata;
pub mod sequencing_run;
pub mod index_sets;

#[cfg(feature = "python")]
mod python_modules {
    use pyo3::prelude::*;
    use crate::institution::*;

    #[pymodule]
    fn scamplers_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_class::<NewInstitution>()?;
        m.add_class::<Institution>()?;

        Ok(())
    }
}
