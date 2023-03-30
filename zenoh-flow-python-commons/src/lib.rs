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

#![allow(clippy::borrow_deref_ref)]
// This allow is needed for a false positive
// when using &PyBytes as function parameter.

use pyo3::exceptions::{PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList, PyLong, PyString};
use std::convert::{TryFrom, TryInto};
use zenoh_flow::bail;

use zenoh_flow::prelude::{
    zferror, Configuration, Context as ZFContext, Error, ErrorKind, InputRaw as ZInput, Inputs,
    OutputRaw as ZOutput, Outputs, Payload,
};
use zenoh_flow::types::LinkMessage as ZFMessage;

use std::sync::Arc;

#[derive(Clone)]
pub struct PythonState {
    pub module: Arc<PyObject>,
    pub py_state: Arc<PyObject>,
    pub event_loop: Arc<PyObject>,
    pub asyncio_module: Arc<PyObject>,
}

impl Drop for PythonState {
    fn drop(&mut self) {
        let gil = Python::acquire_gil();
        let py = gil.python();

        let py_op = self
            .py_state
            .cast_as::<PyAny>(py)
            .expect("Unable to get Python Node module!");

        py_op
            .call_method0("finalize")
            .expect("Unable to call Python finalize!");
    }
}

unsafe impl Send for PythonState {}
unsafe impl Sync for PythonState {}

impl std::fmt::Debug for PythonState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PythonState").finish()
    }
}

pub fn from_pyerr_to_zferr(py_err: pyo3::PyErr, py: &pyo3::Python<'_>) -> Error {
    let tb = if let Some(traceback) = py_err.traceback(*py) {
        traceback.format().map_or_else(|_| "".to_string(), |s| s)
    } else {
        "".to_string()
    };

    zferror!(
        ErrorKind::InvalidData,
        "Error: {:?}\nTraceback: {:?}",
        py_err,
        tb
    )
    .into()
}

pub fn from_pydwncasterr_to_zferr(py_err: pyo3::PyDowncastError) -> Error {
    zferror!(ErrorKind::InvalidData, "Error: {:?}", py_err,).into()
}

pub fn context_into_py<'p>(py: &'p Python, ctx: &ZFContext) -> PyResult<&'p PyAny> {
    let py_zf_types = PyModule::import(*py, "zenoh_flow.types")?;

    let py_ctx = py_zf_types.getattr("Context")?.call1((
        format!("{}", ctx.get_runtime_name()),
        format!("{}", ctx.get_runtime_uuid()),
        format!("{}", ctx.get_flow_name()),
        format!("{}", ctx.get_instance_id()),
    ))?;

    Ok(py_ctx)
}

pub fn configuration_into_py(py: Python, value: Configuration) -> PyResult<PyObject> {
    match value {
        Configuration::Array(arr) => {
            let py_list = PyList::empty(py);
            for v in arr {
                py_list.append(configuration_into_py(py, v)?)?;
            }
            Ok(py_list.to_object(py))
        }
        Configuration::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (k, v) in obj {
                py_dict.set_item(k, configuration_into_py(py, v)?)?;
            }
            Ok(py_dict.to_object(py))
        }
        Configuration::Bool(b) => Ok(b.to_object(py)),
        Configuration::Number(n) => {
            if n.is_i64() {
                Ok(n.as_i64()
                    .ok_or_else(|| {
                        PyErr::from_value(
                            PyTypeError::new_err(format!("Unable to convert {n:?} to i64"))
                                .value(py),
                        )
                    })?
                    .to_object(py))
            } else if n.is_u64() {
                Ok(n.as_u64()
                    .ok_or_else(|| {
                        PyErr::from_value(
                            PyTypeError::new_err(format!("Unable to convert {n:?} to u64"))
                                .value(py),
                        )
                    })?
                    .to_object(py))
            } else {
                Ok(n.as_f64()
                    .ok_or_else(|| {
                        PyErr::from_value(
                            PyTypeError::new_err(format!("Unable to convert {n:?} to f64"))
                                .value(py),
                        )
                    })?
                    .to_object(py))
            }
        }
        Configuration::String(s) => Ok(s.to_object(py)),
        Configuration::Null => Ok(py.None()),
    }
}

pub fn inputs_into_py(py: Python, mut inputs: Inputs) -> PyResult<PyObject> {
    let py_zenoh_flow = py.import("zenoh_flow")?;

    let py_receivers = PyDict::new(py);
    let inputs_ids = inputs.keys().cloned().collect::<Vec<_>>();
    for id in &inputs_ids {
        let input = inputs.take_raw(id).ok_or(PyValueError::new_err(format!("Unable to find input {id}")))?;

        let pyo3_rx = RawInput::from(input);
        py_receivers.set_item(PyString::new(py, id), &pyo3_rx.into_py(py))?;

    }

    let py_inputs = py_zenoh_flow.getattr("Inputs")?.call1((py_receivers,))?;
    Ok(py_inputs.to_object(py))
}

pub fn outputs_into_py(py: Python, mut outputs: Outputs) -> PyResult<PyObject> {
    let py_zenoh_flow = py.import("zenoh_flow")?;

    let py_senders = PyDict::new(py);
    let outputs_ids = outputs.keys().cloned().collect::<Vec<_>>();
    for id in &outputs_ids {
        let output = outputs.take_raw(id).ok_or(PyValueError::new_err(format!("Unable to find output {id}")))?;
        let pyo3_tx = RawOutput::from(output);
        py_senders.set_item(PyString::new(py, id), &pyo3_tx.into_py(py))?;
    }

    let py_outputs = py_zenoh_flow.getattr("Outputs")?.call1((py_senders,))?;
    Ok(py_outputs.to_object(py))
}

/// Channels that sends data to downstream nodes.
#[pyclass]
pub struct RawOutput {
    pub(crate) sender: Arc<ZOutput>,
}

#[pymethods]
impl RawOutput {
    /// Send, *asynchronously*, the bytes on all channels.
    ///
    /// If no timestamp is provided, the current timestamp — as per the HLC — is taken.
    ///
    /// If an error occurs while sending the message on a channel, we still try to send it on the
    /// remaining channels. For each failing channel, an error is logged and counted for.
    pub fn send<'p>(
        &'p self,
        py: Python<'p>,
        data: &PyBytes,
        ts: Option<u64>,
    ) -> PyResult<&'p PyAny> {
        let c_sender = self.sender.clone();
        let rust_data = Payload::from(data.as_bytes());
        pyo3_asyncio::async_std::future_into_py(py, async move {
            c_sender
                .send(rust_data, ts)
                .await
                .map_err(|_| PyValueError::new_err("Unable to send data"))?;
            Ok(Python::with_gil(|py| py.None()))
        })
    }

    /// Returns the ID associated with this `Output`.
    pub fn port_id<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyString> {
        let port_id = self.sender.port_id();
        Ok(PyString::new(py, port_id))
    }
}

impl From<ZOutput> for RawOutput {
    fn from(other: ZOutput) -> Self {
        Self {
            sender: Arc::new(other),
        }
    }
}

impl From<&ZOutput> for RawOutput {
    fn from(other: &ZOutput) -> Self {
        Self {
            sender: Arc::new(other.clone()),
        }
    }
}

/// Channels that receives data from upstream nodes.
#[pyclass(subclass)]
pub struct RawInput {
    pub(crate) receiver: Arc<ZInput>,
}

#[pymethods]
impl RawInput {
    /// Returns the first `RawDataMessage` that was received, *asynchronously*, on any of the channels
    /// associated with this Input.
    ///
    /// If several `RawDataMessage` are received at the same time, one is randomly selected.
    pub fn recv<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let c_receiver = self.receiver.clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            let rust_msg = c_receiver
                .recv()
                .await
                .map_err(|_| PyValueError::new_err("Unable to receive data"))?;
            RawDataMessage::try_from(rust_msg)
        })
    }

    /// Returns the ID associated with this `Input`.
    pub fn port_id<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyString> {
        let port_id = self.receiver.port_id();
        Ok(PyString::new(py, port_id))
    }
}

impl From<ZInput> for RawInput {
    fn from(other: ZInput) -> Self {
        Self {
            receiver: Arc::new(other),
        }
    }
}

impl From<&ZInput> for RawInput {
    fn from(other: &ZInput) -> Self {
        Self {
            receiver: Arc::new(other.clone()),
        }
    }
}

impl TryInto<ZInput> for RawInput {
    type Error = zenoh_flow::prelude::Error;

    fn try_into(self) -> Result<ZInput, Self::Error> {
        match Arc::try_unwrap(self.receiver) {
            Ok(input) => Ok(input),
            Err(_) => bail!(
                ErrorKind::GenericError,
                "Cannot get Input from Python, maybe using a callback in the iteration function?"
            ),
        }
    }
}

/// Zenoh Flow data messages
/// It contains the actual data, the timestamp associated, and
/// information whether the message is a `Watermark`
#[pyclass(subclass)]
pub struct RawDataMessage {
    data: Py<PyBytes>,
    ts: Py<PyLong>,
    is_watermark: bool,
}

#[pymethods]
impl RawDataMessage {
    /// Creates a new [`RawDataMessage`](`RawDataMessage`) with given bytes,
    ///  timestamp and watermark flag.
    #[new]
    pub fn new(data: Py<PyBytes>, ts: Py<PyLong>, is_watermark: bool) -> Self {
        Self {
            data,
            ts,
            is_watermark,
        }
    }

    /// Returns a reference over bytes representing the data.
    #[getter]
    pub fn get_data(&self) -> &Py<PyBytes> {
        &self.data
    }

    /// Returns the data timestamp.
    #[getter]
    pub fn get_ts(&self) -> &Py<PyLong> {
        &self.ts
    }

    /// Returns whether the `RawDataMessage` is a watermark or not.
    #[getter]
    pub fn is_watermark(&self) -> bool {
        self.is_watermark
    }
}

impl TryFrom<ZFMessage> for RawDataMessage {
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
        }
    }
}
