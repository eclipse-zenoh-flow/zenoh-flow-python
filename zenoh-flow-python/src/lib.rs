//
// Copyright (c) 2017, 2021 ADLINK Technology Inc.
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ADLINK zenoh team, <zenoh@adlink-labs.tech>
//

use pyo3::types::PyDict;
use pyo3::{prelude::*, types::PyModule};

pub use zenoh_flow_python_types::{Context, Inputs, Outputs};

#[pyclass(subclass)]
pub struct Source {}

#[pymethods]
impl Source {
    #[new]
    fn new() -> Self {
        Source {}
    }

    fn run(&self, _state: Py<PyAny>) -> PyResult<Vec<u8>> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }

    fn initialize(&self, _configuration: Option<PyObject>) -> PyResult<Py<PyAny>> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }

    fn finalize(&self, _state: Py<PyAny>) -> PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }
}

#[pyclass(subclass)]
pub struct Sink {}

#[pymethods]
impl Sink {
    #[new]
    fn new() -> Self {
        Self {}
    }

    fn run(&self, _context: &mut Context, _state: Py<PyAny>, _input: Vec<u8>) -> PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }

    fn initialize(&self, _configuration: Option<PyObject>) -> PyResult<Py<PyAny>> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }

    fn finalize(&self, _state: Py<PyAny>) -> PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }
}

#[pyclass(subclass)]
pub struct Operator {}

#[pymethods]
impl Operator {
    #[new]
    fn new() -> Self {
        Self {}
    }

    fn input_rule(
        &self,
        _context: &mut Context,
        _state: Py<PyAny>,
        _tokens: Py<PyDict>,
    ) -> PyResult<bool> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }

    fn run(
        &self,
        _context: &mut Context,
        _state: Py<PyAny>,
        _inputs: Inputs,
    ) -> PyResult<Py<PyDict>> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }

    fn output_rule(
        &self,
        _context: &mut Context,
        _state: Py<PyAny>,
        _outputs: Outputs,
    ) -> PyResult<Outputs> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }

    fn initialize(&self, _configuration: Option<PyObject>) -> PyResult<Py<PyAny>> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }

    fn finalize(&self, _state: Py<PyAny>) -> PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }
}

#[pymodule]
fn zenoh_flow(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Source>()?;
    m.add_class::<Sink>()?;
    m.add_class::<Operator>()?;
    m.add_class::<Inputs>()?;
    m.add_class::<Outputs>()?;
    m.add_class::<Context>()?;

    Ok(())
}
