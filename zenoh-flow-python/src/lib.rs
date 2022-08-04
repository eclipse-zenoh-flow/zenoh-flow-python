use pyo3::prelude::*;
use zenoh_flow_python_common::{DataMessage, DataReceiver, DataSender};

#[pymodule]
fn zenoh_flow(_py: Python, m: &PyModule) -> PyResult<()> {
    // m.add_class::<PyReceiver>()?;
    // m.add_class::<PySender>()?;
    m.add_class::<DataSender>()?;
    m.add_class::<DataReceiver>()?;
    m.add_class::<DataMessage>()?;

    Ok(())
}
