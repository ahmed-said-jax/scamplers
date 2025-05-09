pub mod chromium;
mod dataset_metadata;
pub mod index_sets;
pub mod institution;
pub mod lab;
pub mod person;
mod sample_metadata;
pub mod sequencing_run;

#[cfg(feature = "python")]
mod python_modules {
    use crate::institution::*;
    use pyo3::prelude::*;

    #[pymodule]
    fn scamplers_core(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_class::<NewInstitution>()?;
        m.add_class::<Institution>()?;

        Ok(())
    }
}
