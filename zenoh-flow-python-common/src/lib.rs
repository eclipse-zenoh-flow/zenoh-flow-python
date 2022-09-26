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
use pyo3::types::{PyBytes, PyDict, PyList, PyLong, PyString, PyTuple};
use std::convert::{TryFrom, TryInto};
use zenoh_flow::bail;
use zenoh_flow::types::Streams;

use zenoh_flow::prelude::{
    zferror, Configuration, Context as ZFContext, Data, Error, ErrorKind, Input, Inputs,
    Message as ZFMessage, Output, Outputs, Result as ZFResult,
};
use zenoh_flow::traits::{InputCallback, OutputCallback};

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
            .module
            .cast_as::<PyAny>(py)
            .expect("Unable to get Python Node module!");

        py_op
            .call_method1("finalize", (py_op,))
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

pub fn get_python_input_callbacks(
    py: &Python,
    py_context: &PyAny,
    mut inputs: Inputs,
) -> ZFResult<Vec<(Input, Box<dyn InputCallback>)>> {
    let mut input_callbacks = Vec::new();

    let py_input_cbs: &PyDict = py_context
        .getattr("input_callbacks")
        .map_err(|e| from_pyerr_to_zferr(e, &py))?
        .cast_as()
        .map_err(|e| from_pydwncasterr_to_zferr(e))?;

    for (key, value) in py_input_cbs.iter() {
        let input_id = key
            .cast_as::<PyString>()
            .map_err(|e| from_pydwncasterr_to_zferr(e))?
            .to_str()
            .map_err(|e| from_pyerr_to_zferr(e, &py))?;

        let owned_callback = value.to_object(*py);

        match inputs.take(input_id) {
            Some(input) => {
                let rust_callback = py_input_sync_callback_to_rust(owned_callback);
                input_callbacks.push((input, rust_callback));
            }
            None => {
                bail!(ErrorKind::NotFound, "Input {:?} not found", input_id)
            }
        }
    }

    Ok(input_callbacks)
}

pub fn py_input_sync_callback_to_rust(callback_py: Py<PyAny>) -> Box<dyn InputCallback> {
    Box::new(move |msg: ZFMessage| {
        let callback_py = callback_py.clone();

        async move {
            let py_msg = DataMessage::try_from(msg)?;
            Python::with_gil(|py| callback_py.call1(py, (py_msg,)))
                .map_err(|e| Python::with_gil(|py| from_pyerr_to_zferr(e, &py)))?;
            Ok(())
        }
    })
}

pub fn get_python_output_callbacks(
    py: &Python,
    py_context: &PyAny,
    mut outputs: Outputs,
) -> ZFResult<Vec<(Output, Box<dyn OutputCallback>)>> {
    let mut output_callbacks = Vec::new();

    let py_input_cbs: &PyDict = py_context
        .getattr("output_callbacks")
        .map_err(|e| from_pyerr_to_zferr(e, &py))?
        .cast_as()
        .map_err(|e| from_pydwncasterr_to_zferr(e))?;

    for (key, value) in py_input_cbs.iter() {
        let output_id = key
            .cast_as::<PyString>()
            .map_err(|e| from_pydwncasterr_to_zferr(e))?
            .to_str()
            .map_err(|e| from_pyerr_to_zferr(e, &py))?;

        let value = value
            .cast_as::<PyTuple>()
            .map_err(|e| from_pydwncasterr_to_zferr(e))?;

        let owned_callback = value
            .get_item(0)
            .map_err(|e| from_pyerr_to_zferr(e, &py))?
            .to_object(*py);
        let timeout_ms: u64 = value
            .get_item(1)
            .map_err(|e| from_pyerr_to_zferr(e, &py))?
            .extract()
            .map_err(|e| from_pyerr_to_zferr(e, &py))?;

        match outputs.take(output_id) {
            Some(output) => {
                let rust_callback = py_output_sync_callback_to_rust(owned_callback, timeout_ms);
                output_callbacks.push((output, rust_callback));
            }
            None => {
                bail!(ErrorKind::NotFound, "Output {:?} not found", output_id)
            }
        }
    }

    Ok(output_callbacks)
}

pub fn py_output_sync_callback_to_rust(
    callback_py: Py<PyAny>,
    timeout_ms: u64,
) -> Box<dyn OutputCallback> {
    Box::new(move || {
        let callback_py = callback_py.clone();
        async move {
            // Sleeping in rust to avoid locks on Python GIL
            async_std::task::sleep(std::time::Duration::from_millis(timeout_ms)).await;

            let data_bytes = Python::with_gil(|py| {
                let callback_result = callback_py.call0(py)?;
                let bytes = callback_result.cast_as::<PyBytes>(py)?;
                Ok(bytes.as_bytes().to_vec())
            })
            .map_err(|e| Python::with_gil(|py| from_pyerr_to_zferr(e, &py)))?;

            Ok(Data::from(data_bytes))
        }
    })
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
        Configuration::String(s) => Ok(s.to_object(py)),
        Configuration::Null => Ok(py.None()),
    }
}

/// Channels that sends data to downstream nodes.
#[pyclass]
pub struct DataSender {
    pub(crate) sender: Arc<Output>,
}

#[pymethods]
impl DataSender {
    /// Send, *asynchronously*, the `DataMessage` on all channels.
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
        let rust_data = Data::from(data.as_bytes());
        pyo3_asyncio::async_std::future_into_py(py, async move {
            c_sender
                .send_async(rust_data, ts)
                .await
                .map_err(|_| PyValueError::new_err("Unable to send data"))?;
            Ok(Python::with_gil(|py| py.None()))
        })
    }

    /// Returns the ID associated with this `DataSender`.
    pub fn port_id<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyString> {
        //@FIXME: this should be updated once
        // https://github.com/eclipse-zenoh/zenoh-flow/issues/122
        // is fixed.
        let port_id = self.sender.port_id();
        Ok(PyString::new(py, port_id))
    }
}

impl From<Output> for DataSender {
    fn from(other: Output) -> Self {
        Self {
            sender: Arc::new(other.clone()),
        }
    }
}

impl From<&Output> for DataSender {
    fn from(other: &Output) -> Self {
        Self {
            sender: Arc::new(other.clone()),
        }
    }
}

/// Channels that receives data from upstream nodes.
#[pyclass(subclass)]
pub struct DataReceiver {
    pub(crate) receiver: Arc<Input>,
}

#[pymethods]
impl DataReceiver {
    /// Returns the first `DataMessage` that was received, *asynchronously*, on any of the channels
    /// associated with this DataReceiver.
    ///
    /// If several `DataMessage` are received at the same time, one is randomly selected.
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

    /// Returns the ID associated with this `DataReceiver`.
    pub fn port_id<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyString> {
        //@FIXME: this should be updated once
        // https://github.com/eclipse-zenoh/zenoh-flow/issues/122
        // is fixed.
        let port_id = self.receiver.id();
        Ok(PyString::new(py, port_id))
    }
}

impl From<Input> for DataReceiver {
    fn from(other: Input) -> Self {
        Self {
            receiver: Arc::new(other.clone()),
        }
    }
}

impl From<&Input> for DataReceiver {
    fn from(other: &Input) -> Self {
        Self {
            receiver: Arc::new(other.clone()),
        }
    }
}

impl TryInto<Input> for DataReceiver {
    type Error = zenoh_flow::prelude::Error;

    fn try_into(self) -> Result<Input, Self::Error> {
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
pub struct DataMessage {
    data: Py<PyBytes>,
    ts: Py<PyLong>,
    is_watermark: bool,
}

#[pymethods]
impl DataMessage {
    /// Creates a new [`DataMessage`](`DataMessage`) with given bytes,
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

    /// Returns whether the `DataMessage` is a watermark or not.
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
