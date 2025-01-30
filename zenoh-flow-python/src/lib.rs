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

use std::{fmt::Display, path::PathBuf, sync::Arc};

use anyhow::anyhow;
use pyo3::{
    exceptions::{PyTypeError, PyValueError},
    prelude::*,
    types::{PyDict, PyList},
    PyErr, PyObject, PyResult, Python, ToPyObject,
};
use pyo3_asyncio::TaskLocals;
use serde_json::Value;
use zenoh_flow_nodes::prelude as zf;

#[derive(Clone)]
pub struct PythonState {
    pub node_instance: Arc<PyObject>,
    pub task_locals: Arc<TaskLocals>,
}

impl Drop for PythonState {
    fn drop(&mut self) {
        Python::with_gil(|py| {
            self.node_instance
                .call_method0(py, "finalize")
                .expect("Failed to call `finalize` on the internal Python state");
        });
    }
}

#[pymodule]
fn zenoh_flow_python(_py: Python<'_>, m: &PyModule) -> PyResult<()> {
    m.add_class::<InstanceId>()?;
    m.add_class::<RuntimeId>()?;
    m.add_class::<Context>()?;
    m.add_class::<Timestamp>()?;
    m.add_class::<LinkMessage>()?;
    m.add_class::<InputRaw>()?;
    m.add_class::<Inputs>()?;
    Ok(())
}

#[pyo3::pyclass]
#[derive(Debug)]
pub struct InstanceId(pub(crate) zf::InstanceId);

impl From<&zf::InstanceId> for InstanceId {
    fn from(value: &zf::InstanceId) -> Self {
        Self(value.clone())
    }
}

impl Display for InstanceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0.simple())
    }
}

#[pyo3::pyclass]
#[derive(Debug)]
pub struct RuntimeId(pub(crate) zf::RuntimeId);

impl From<&zf::RuntimeId> for RuntimeId {
    fn from(value: &zf::RuntimeId) -> Self {
        Self(value.clone())
    }
}

impl Display for RuntimeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[pyo3::pyclass]
pub struct Context(pub(crate) zf::Context);

impl From<zf::Context> for Context {
    fn from(value: zf::Context) -> Self {
        Self(value)
    }
}

#[pyo3::pymethods]
impl Context {
    pub fn data_flow_name(&self) -> &str {
        self.0.name()
    }

    pub fn data_flow_instance_id(&self) -> InstanceId {
        self.0.instance_id().into()
    }

    pub fn runtime_id(&self) -> RuntimeId {
        self.0.runtime_id().into()
    }

    pub fn library_path(&self) -> &PathBuf {
        self.0.library_path()
    }

    pub fn node_id(&self) -> &str {
        self.0.node_id()
    }
}

#[pyo3::pyclass]
pub struct Timestamp(pub(crate) zf::Timestamp);

impl From<&zf::Timestamp> for Timestamp {
    fn from(value: &zf::Timestamp) -> Self {
        Timestamp(*value)
    }
}

#[pyo3::pymethods]
impl Timestamp {
    fn time(&self) -> u64 {
        // NOTE: u64::MAX, if converted from **nanoseconds** to a timestamp, will cease to function in 2554. Given we're
        // converting to **milliseconds** we should be safe.
        self.0.get_time().to_duration().as_millis() as u64
    }

    fn id(&self) -> String {
        self.0.get_id().to_string()
    }
}

#[pyo3::pyclass]
pub struct LinkMessage {
    pub(crate) message: zf::LinkMessage,
    pub(crate) serialised_payload: Vec<u8>,
}

impl From<zf::LinkMessage> for LinkMessage {
    fn from(value: zf::LinkMessage) -> Self {
        LinkMessage {
            message: value,
            serialised_payload: Vec::default(),
        }
    }
}

#[pyo3::pymethods]
impl LinkMessage {
    pub fn payload(&mut self) -> PyResult<&[u8]> {
        match self.message.payload() {
            // TODO As pointed by @gabrik, in a previous version of Zenoh-Flow the call to the
            //      method `as_slice` was replaced with another approach because, performance-wise,
            //      it was vastly inferior.
            //
            //      Given that pyO3 changed quite a bit between these versions, we ought to check
            //      if that assumption still holds.
            zf::Payload::Bytes(bytes) => Ok(bytes.as_slice()),
            zf::Payload::Typed((data, serialiser)) => {
                if self.serialised_payload.is_empty() {
                    (serialiser)(&mut self.serialised_payload, data.clone()).map_err(|e| {
                        ZFError::from(anyhow!("Failed to serialise payload: {e:?}"))
                    })?;
                }

                Ok(self.serialised_payload.as_slice())
            }
        }
    }

    pub fn timestamp(&self) -> Timestamp {
        self.message.timestamp().into()
    }
}

struct ZFError(pub(crate) anyhow::Error);

impl From<ZFError> for PyErr {
    fn from(value: ZFError) -> Self {
        let message_chain = value
            .0
            .chain()
            .map(|e| e.to_string())
            .fold(String::default(), |acc, e| format!("{acc}\n{e}"));

        PyValueError::new_err(message_chain)
    }
}

impl From<anyhow::Error> for ZFError {
    fn from(value: anyhow::Error) -> Self {
        Self(value)
    }
}

/// Converts the provided [Configuration] into a [PyObject].
///
/// This function is required because we cannot simply wrap a [Configuration] in a new type and then expose the same
/// methods as [serde_json::Value].
pub fn configuration_into_py(
    py: Python,
    mut configuration: zf::Configuration,
) -> PyResult<PyObject> {
    match configuration.take() {
        Value::Array(arr) => {
            let py_list = PyList::empty(py);
            for v in arr {
                py_list.append(configuration_into_py(py, v.into())?)?;
            }
            Ok(py_list.to_object(py))
        }
        Value::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (k, v) in obj {
                py_dict.set_item(k, configuration_into_py(py, v.into())?)?;
            }
            Ok(py_dict.to_object(py))
        }
        Value::Bool(b) => Ok(b.to_object(py)),
        Value::Number(n) => {
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
        Value::String(s) => Ok(s.to_object(py)),
        Value::Null => Ok(py.None()),
    }
}

#[pyo3::pyclass]
pub struct InputRaw(pub(crate) zf::InputRaw);

impl From<zf::InputRaw> for InputRaw {
    fn from(input: zf::InputRaw) -> Self {
        Self(input)
    }
}

#[pyo3::pymethods]
impl InputRaw {
    pub fn recv_async<'p>(&'p self, py: Python<'p>) -> PyResult<&'p PyAny> {
        let this = self.0.clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            match this.recv().await {
                Ok(link_message) => Ok(LinkMessage::from(link_message)),
                Err(e) => Err(ZFError::from(e).into()),
            }
        })
    }

    pub fn try_recv(&self) -> PyResult<Option<LinkMessage>> {
        match self.0.try_recv() {
            Ok(maybe_message) => Ok(maybe_message.map(LinkMessage::from)),
            Err(e) => Err(ZFError::from(e).into()),
        }
    }

    pub fn port_id(&self) -> &str {
        self.0.port_id()
    }
}

#[pyo3::pyclass]
pub struct Inputs(pub(crate) zf::Inputs);

impl From<zf::Inputs> for Inputs {
    fn from(value: zf::Inputs) -> Self {
        Self(value)
    }
}

#[pyo3::pymethods]
impl Inputs {
    pub fn take_raw(&mut self, port_id: &str) -> PyResult<InputRaw> {
        self.0
            .take(port_id)
            .map(|input_builder| input_builder.raw().into())
            .ok_or_else(|| {
                ZFError::from(anyhow!("Found no Input associated with port < {port_id} >")).into()
            })
    }
}

#[pyo3::pyclass]
pub struct OutputRaw(pub(crate) zf::OutputRaw);

impl From<zf::OutputRaw> for OutputRaw {
    fn from(output: zf::OutputRaw) -> Self {
        Self(output)
    }
}

#[pyo3::pymethods]
impl OutputRaw {
    pub fn send_async<'p>(
        &'p self,
        py: Python<'p>,
        payload: Vec<u8>,
        timestamp: Option<u64>,
    ) -> PyResult<&'p PyAny> {
        let this = self.0.clone();
        let port_id = self.0.port_id().clone();
        pyo3_asyncio::async_std::future_into_py(py, async move {
            this.send(payload, timestamp).await.map_err(|e| {
                ZFError::from(anyhow!("Failed to send on < {port_id} >: {e:?}")).into()
            })
        })
    }

    pub fn try_send(&self, payload: &[u8], timestamp: Option<u64>) -> PyResult<()> {
        self.0.try_send(payload, timestamp).map_err(|e| {
            ZFError::from(anyhow!(
                "Call to `try_send` on < {} > failed with: {e:?}",
                self.0.port_id()
            ))
            .into()
        })
    }

    pub fn port_id(&self) -> &str {
        self.0.port_id()
    }
}

#[pyo3::pyclass]
pub struct Outputs(pub(crate) zf::Outputs);

impl From<zf::Outputs> for Outputs {
    fn from(outputs: zf::Outputs) -> Self {
        Self(outputs)
    }
}

#[pyo3::pymethods]
impl Outputs {
    pub fn take_raw(&mut self, port_id: &str) -> PyResult<OutputRaw> {
        self.0
            .take(port_id)
            .map(|output_builder| output_builder.raw().into())
            .ok_or_else(|| {
                ZFError::from(anyhow!(
                    "Found not Output associated with port < {port_id} >"
                ))
                .into()
            })
    }
}
