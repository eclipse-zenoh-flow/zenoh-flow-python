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
use pyo3::types::{PyBool, PyBytes, PyDict, PyInt, PyList, PyLong, PyString};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::time::Duration;
use uhlc::{Timestamp, NTP64};
use uuid::Uuid;
use zenoh_flow::model::{InputDescriptor, OutputDescriptor};
use zenoh_flow::runtime::message::DataMessage as ZFDataMessage;
use zenoh_flow::types::{Data, NodeOutput};
use zenoh_flow::{Context, Input, Message as ZFMessage, Output, PortId, ZFError, ZFResult};

use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::zenoh_flow_derive::ZFState;

#[derive(ZFState, Clone)]
pub struct PythonState {
    pub module: Arc<PyObject>,
    pub py_state: Arc<PyObject>,
    pub py_zf_types: Arc<PyObject>,
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
        format!(
            "{}",
            traceback.format().map_or_else(|_| "".to_string(), |s| s)
        )
    } else {
        "".to_string()
    };

    let err_str = format!("Error: {:?}\nTraceback: {:?}", py_err, tb);
    ZFError::InvalidData(err_str)
}

pub fn from_pyerr_to_zferr_no_trace(py_err: pyo3::PyErr) -> ZFError {
    let err_str = format!("Error: {:?}\n", py_err);
    ZFError::InvalidData(err_str)
}

/// Converts rust `Context` into a `PyAny` to be passed to Python.
// pub fn from_context_to_pyany<'a>(
//     ctx: &'_ Context,
//     py: &'a Python,
//     zf_types_module: &'a PyModule,
// ) -> ZFResult<&'a PyAny> {
//     zf_types_module
//         .getattr("Context")
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .call1((ctx.mode,))
//         .map_err(|e| from_pyerr_to_zferr(e, py))
// }

// /// Converts Python `PyAny` into a rust `Context`.
// pub fn from_pyany_to_context<'a>(from: &'a PyAny, py: &'a Python) -> ZFResult<Context> {
//     let mode: usize = from
//         .call_method0("get_mode")
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .extract()
//         .map_err(|e| from_pyerr_to_zferr(e.into(), py))?;

//     Ok(Context { mode })
// }

// /// Converts rust `OutputDescriptor` into a `PyAny` to be passed to Python.
// pub fn from_output_descriptor_to_pyany<'a>(
//     from: &'_ OutputDescriptor,
//     py: &'a Python,
//     zf_types_module: &'a PyModule,
// ) -> ZFResult<&'a PyAny> {
//     zf_types_module
//         .getattr("FromDescriptor")
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .call1((
//             PyString::new(*py, &from.node),
//             PyString::new(*py, &from.output),
//         ))
//         .map_err(|e| from_pyerr_to_zferr(e, py))
// }
// /// Converts Python `PyAny` into a rust `OutputDescriptor`.
// pub fn from_pyany_to_output_descritptor<'a>(
//     from: &'a PyAny,
//     py: &'a Python,
// ) -> ZFResult<OutputDescriptor> {
//     let node = from
//         .getattr("node")
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .cast_as::<PyString>()
//         .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
//         .to_str()
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .to_string()
//         .into();

//     let output = from
//         .getattr("output")
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .cast_as::<PyString>()
//         .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
//         .to_str()
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .to_string()
//         .into();

//     Ok(OutputDescriptor { node, output })
// }

// /// Converts rust `InputDescriptor` into a `PyAny` to be passed to Python.
// pub fn from_input_descriptor_to_pyany<'a>(
//     from: &'_ InputDescriptor,
//     py: &'a Python,
//     zf_types_module: &'a PyModule,
// ) -> ZFResult<&'a PyAny> {
//     zf_types_module
//         .getattr("FromDescriptor")
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .call1((
//             PyString::new(*py, &from.node),
//             PyString::new(*py, &from.input),
//         ))
//         .map_err(|e| from_pyerr_to_zferr(e, py))
// }
// /// Converts Python `PyAny` into a rust `InputDescriptor`.
// pub fn from_pyany_to_input_descritptor<'a>(
//     from: &'a PyAny,
//     py: &'a Python,
// ) -> ZFResult<InputDescriptor> {
//     let node = from
//         .getattr("node")
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .cast_as::<PyString>()
//         .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
//         .to_str()
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .to_string()
//         .into();

//     let input = from
//         .getattr("input")
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .cast_as::<PyString>()
//         .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
//         .to_str()
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .to_string()
//         .into();

//     Ok(InputDescriptor { node, input })
// }

// /// Converts rust `Timestamp` into a `PyAny` to be passed to Python.
// pub fn from_timestamp_to_pyany<'a>(
//     from: &'_ Timestamp,
//     py: &'a Python,
//     zf_types_module: &'a PyModule,
// ) -> ZFResult<&'a PyAny> {
//     zf_types_module
//         .getattr("Timestamp")
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .call1((
//             from.get_time().as_u64(),
//             PyString::new(*py, &from.get_id().to_string()),
//         ))
//         .map_err(|e| from_pyerr_to_zferr(e, py))
// }

// /// Converts Python `PyAny` into a rust `Timestamp`.
// pub fn from_pyany_to_timestamp<'a>(from: &'a PyAny, py: &'a Python) -> ZFResult<Timestamp> {
//     let ntp: u64 = from
//         .getattr("ntp")
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .extract()
//         .map_err(|e| from_pyerr_to_zferr(e.into(), py))?;

//     let id = from
//         .getattr("id")
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .cast_as::<PyString>()
//         .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
//         .to_str()
//         .map_err(|e| from_pyerr_to_zferr(e, py))?;

//     let id = Uuid::parse_str(id).map_err(|_| ZFError::DeseralizationError)?;
//     Ok(Timestamp::new(NTP64(ntp), id.into()))
// }

// /// Converts rust `DataMessage` into a `PyAny` to be passed to Python.
// pub fn from_data_message_to_pyany<'a>(
//     from: &'_ mut DataMessage,
//     py: &'a Python,
//     zf_types_module: &'a PyModule,
// ) -> ZFResult<&'a PyAny> {
//     // let pye2e = PyList::empty(*py);

//     // for e in from.get_missed_end_to_end_deadlines() {
//     //     pye2e
//     //         .append(from_e2e_deadline_miss_to_pyany(e, py, zf_types_module)?)
//     //         .map_err(|e| from_pyerr_to_zferr(e, py))?;
//     // }

//     let py_it = zf_types_module
//         .getattr("DataMessage")
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .call1((
//             from_timestamp_to_pyany(from.get_timestamp(), py, zf_types_module)?,
//             PyBytes::new(*py, &from.get_inner_data().try_as_bytes()?),
//         ))
//         .map_err(|e| from_pyerr_to_zferr(e, py))?;

//     Ok(py_it)
// }

// /// Converts Python `PyAny` into a rust `DataMessage`.
// pub fn from_pyany_to_data_message<'a>(from: &'a PyAny, py: &'a Python) -> ZFResult<DataMessage> {
//     let data = from
//         .getattr("data")
//         .map_err(|e| from_pyerr_to_zferr(e, py))?
//         .cast_as::<PyBytes>()
//         .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
//         .as_bytes()
//         .to_vec();

//     let ts = from_pyany_to_timestamp(
//         from.getattr("data")
//             .map_err(|e| from_pyerr_to_zferr(e, py))?,
//         py,
//     )?;

//     // let mut e2ed = vec![];
//     // let pye2ed = from.getattr("missed_end_to_end_deadlines")
//     //     .map_err(|e| from_pyerr_to_zferr(e, py))?;

//     // for e in pye2ed.iter() {
//     //     e2ed.push(from_pyany_to_e2e_deadline_miss(e, py)?);
//     // }

//     Ok(DataMessage::new(Data::from_bytes(data), ts))
// }

// /// Converts rust `HashMap<PortId, DataMessage>` into a `PyDict` to be passed to Python.
// pub fn from_inputs_to_pydict<'a>(
//     from: &'_ mut HashMap<PortId, DataMessage>,
//     py: &'a Python,
//     zf_types_module: &'a PyModule,
// ) -> ZFResult<&'a PyDict> {
//     let pydict = PyDict::new(*py);

//     for (k, v) in from {
//         let data = from_data_message_to_pyany(v, py, zf_types_module)?;

//         pydict
//             .set_item(PyString::new(*py, k), data)
//             .map_err(|e| from_pyerr_to_zferr(e, py))?
//     }
//     Ok(pydict)
// }

// /// Converts Python `PyAny` into a rust `Data`.
// pub fn from_pyany_to_data<'a>(from: &'a PyAny, py: &'a Python) -> ZFResult<Data> {
//     Ok(Data::from_bytes(
//         from.cast_as::<PyBytes>()
//             .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
//             .as_bytes()
//             .to_vec(),
//     ))
// }

// /// Converts rust `DataMessage` into a `PyAny` to be passed to Python.
// pub fn from_data_to_pybytes<'a>(from: &'_ mut Data, py: &'a Python) -> ZFResult<&'a PyBytes> {
//     Ok(PyBytes::new(*py, &from.try_as_bytes()?))
// }

// /// Converts rust `HashMap<PortId, Data>` into a `PyDict` to be passed to Python.
// pub fn from_outputs_to_pydict<'a>(
//     from: &'_ mut HashMap<PortId, Data>,
//     py: &'a Python,
// ) -> ZFResult<&'a PyDict> {
//     let pydict = PyDict::new(*py);

//     for (k, v) in from {
//         let data = from_data_to_pybytes(v, py)?;

//         pydict
//             .set_item(PyString::new(*py, k), data)
//             .map_err(|e| from_pyerr_to_zferr(e, py))?
//     }
//     Ok(pydict)
// }

// /// Converts Python `PyDict` into a rust `HashMap<PortId, NodeOutput>`.
// pub fn from_pyany_to_or_result<'a>(
//     from: &'a PyAny,
//     py: &'a Python,
// ) -> ZFResult<HashMap<PortId, NodeOutput>> {
//     let pydict = from
//         .cast_as::<PyDict>()
//         .map_err(|e| from_pyerr_to_zferr(e.into(), py))?;

//     let mut outputs = HashMap::with_capacity(pydict.len());

//     for (k, v) in pydict.iter() {
//         let port_id = k
//             .cast_as::<PyString>()
//             .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
//             .to_str()
//             .map_err(|e| from_pyerr_to_zferr(e, py))?
//             .to_string();

//         let data = NodeOutput::Data(Data::from_bytes(
//             v.cast_as::<PyBytes>()
//                 .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
//                 .as_bytes()
//                 .to_vec(),
//         ));

//         outputs.insert(port_id.into(), data);
//     }

//     Ok(outputs)
// }

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

async fn rust_sleep() {
    zenoh_flow::async_std::task::sleep(std::time::Duration::from_millis(500)).await;
}

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
                .send(rust_data, ts)
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
                .recv()
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
        return &self.data;
    }

    #[getter]
    pub fn get_ts(&self) -> &Py<PyLong> {
        return &self.ts;
    }

    #[getter]
    pub fn is_watermark(&self) -> bool {
        return self.is_watermark;
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
