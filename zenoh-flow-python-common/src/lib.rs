//
// Copyright (c) 2022 ZettaScale Technology
//
// This program and the accompanying materials are made available under the
// terms of the Eclipse Public License 2.0 which is available at
// http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
// which is available at https://www.apache.org/licenses/LICENSE-2.0.
//
// SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
//
// Contributors:
//   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
//

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList, PyLong};
use std::convert::TryFrom;
use zenoh_flow::types::Data;
use zenoh_flow::{Input, Message as ZFMessage, Output, ZFError};

use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::zenoh_flow_derive::ZFState;

#[derive(ZFState, Clone)]
pub struct PythonState {
    pub module: Arc<PyObject>,
    pub py_state: Arc<PyObject>,
    pub event_loop: Arc<PyObject>,
    pub asyncio_module: Arc<PyObject>,
}
unsafe impl Send for PythonState {}
unsafe impl Sync for PythonState {}

impl std::fmt::Debug for PythonState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PythonState").finish()
    }
}

pub fn from_pyerr_to_zferr(py_err: pyo3::PyErr, py: &pyo3::Python<'_>) -> ZFError {
    let tb = if let Some(traceback) = py_err.traceback(*py) {
        traceback.format().map_or_else(|_| "".to_string(), |s| s)
    } else {
        "".to_string()
    };

    let err_str = format!("Error: {:?}\nTraceback: {:?}", py_err, tb);
    ZFError::InvalidData(err_str)
}

pub fn configuration_into_py(py: Python, value: zenoh_flow::Configuration) -> PyResult<PyObject> {
    match value {
        zenoh_flow::Configuration::Array(arr) => {
            let py_list = PyList::empty(py);
            for v in arr {
                py_list.append(configuration_into_py(py, v)?)?;
            }
            Ok(py_list.to_object(py))
        }
        zenoh_flow::Configuration::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (k, v) in obj {
                py_dict.set_item(k, configuration_into_py(py, v)?)?;
            }
            Ok(py_dict.to_object(py))
        }
        zenoh_flow::Configuration::Bool(b) => Ok(b.to_object(py)),
        zenoh_flow::Configuration::Number(n) => {
            if n.is_i64() {
                Ok(n.as_i64()
                    .ok_or_else(|| {
                        PyErr::from_value(
                            PyTypeError::new_err(format!("Unable to convert {:?} to i64", n))
                                .value(py),
                        )
                    })?
                    .to_object(py))
            } else if n.is_u64() {
                Ok(n.as_u64()
                    .ok_or_else(|| {
                        PyErr::from_value(
                            PyTypeError::new_err(format!("Unable to convert {:?} to u64", n))
                                .value(py),
                        )
                    })?
                    .to_object(py))
            } else {
                Ok(n.as_f64()
                    .ok_or_else(|| {
                        PyErr::from_value(
                            PyTypeError::new_err(format!("Unable to convert {:?} to f64", n))
                                .value(py),
                        )
                    })?
                    .to_object(py))
            }
        }
        zenoh_flow::Configuration::String(s) => Ok(s.to_object(py)),
        zenoh_flow::Configuration::Null => Ok(py.None()),
    }
}

#[pyclass]
pub struct DataSender {
    pub(crate) sender: Arc<Output>,
}

// unsafe impl Send for DataSender {}
// unsafe impl Sync for DataSender {}

#[pymethods]
impl DataSender {
    pub fn send<'p>(
        &'p self,
        py: Python<'p>,
        data: &PyBytes,
        ts: Option<u64>,
    ) -> PyResult<&'p PyAny> {
        let c_sender = self.sender.clone();
        let rust_data = Data::from_bytes(data.as_bytes().to_owned());
        pyo3_asyncio::async_std::future_into_py(py, async move {
            c_sender
                .send_async(rust_data, ts)
                .await
                .map_err(|_| PyValueError::new_err("Unable to send data"))?;
            Ok(Python::with_gil(|py| py.None()))
        })
    }
}

impl From<Output> for DataSender {
    fn from(other: Output) -> Self {
        Self {
            sender: Arc::new(other),
        }
    }
}

#[pyclass(subclass)]
pub struct DataReceiver {
    pub(crate) receiver: Arc<Input>,
}

// unsafe impl Send for DataReceiver {}
// unsafe impl Sync for DataReceiver {}

#[pymethods]
impl DataReceiver {
    pub fn recv<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let c_receiver = self.receiver.clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            let rust_msg = c_receiver
                .recv_async()
                .await
                .map_err(|_| PyValueError::new_err("Unable to receive data"))?;
            DataMessage::try_from(rust_msg)
        })
    }
}

impl From<Input> for DataReceiver {
    fn from(other: Input) -> Self {
        Self {
            receiver: Arc::new(other),
        }
    }
}

#[pyclass(subclass)]
pub struct DataMessage {
    data: Py<PyBytes>,
    ts: Py<PyLong>,
    is_watermark: bool,
}

#[pymethods]
impl DataMessage {
    #[new]
    pub fn new(data: Py<PyBytes>, ts: Py<PyLong>, is_watermark: bool) -> Self {
        Self {
            data,
            ts,
            is_watermark,
        }
    }

    #[getter]
    pub fn get_data(&self) -> &Py<PyBytes> {
        &self.data
    }

    #[getter]
    pub fn get_ts(&self) -> &Py<PyLong> {
        &self.ts
    }

    #[getter]
    pub fn is_watermark(&self) -> bool {
        self.is_watermark
    }
}

impl TryFrom<ZFMessage> for DataMessage {
    type Error = PyErr;

    fn try_from(other: ZFMessage) -> Result<Self, Self::Error> {
        match other {
            ZFMessage::Data(mut msg) => {
                let data = Python::with_gil(|py| {
                    let bytes = msg
                        .get_inner_data()
                        .try_as_bytes()
                        .map_err(|e| PyValueError::new_err(format!("try_as_bytes field: {e}")))?;

                    Ok::<pyo3::Py<PyBytes>, Self::Error>(Py::from(PyBytes::new(py, bytes.as_ref())))
                })?;

                let ts: Py<PyLong> = Python::with_gil(|py| {
                    Ok::<pyo3::Py<PyLong>, Self::Error>(Py::from(
                        msg.get_timestamp()
                            .get_time()
                            .as_u64()
                            .to_object(py)
                            .cast_as::<PyLong>(py)?,
                    ))
                })?;

                Ok(Self {
                    data,
                    ts,
                    is_watermark: false,
                })
            }
            ZFMessage::Watermark(ts) => {
                let data = Python::with_gil(|py| Py::from(PyBytes::new(py, &[0u8])));
                let ts = Python::with_gil(|py| {
                    Ok::<pyo3::Py<PyLong>, Self::Error>(Py::from(
                        ts.get_time()
                            .as_u64()
                            .to_object(py)
                            .cast_as::<PyLong>(py)
                            .unwrap(),
                    ))
                })?;

                Ok(Self {
                    data,
                    ts,
                    is_watermark: true,
                })
            }
            _ => Err(PyValueError::new_err(
                "Cannot convert ControlMessage to DataMessage",
            )),
        }
    }
}