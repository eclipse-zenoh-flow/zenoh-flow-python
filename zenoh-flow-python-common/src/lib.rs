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

use pyo3::exceptions::PyTypeError;
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList, PyString};
use std::collections::HashMap;
use std::time::Duration;
use uhlc::{Timestamp, NTP64};
use uuid::Uuid;
use zenoh_flow::model::{InputDescriptor, OutputDescriptor};
use zenoh_flow::runtime::deadline::{E2EDeadlineMiss, LocalDeadlineMiss};
use zenoh_flow::runtime::message::DataMessage;
use zenoh_flow::runtime::token::{DataToken, InputToken, TokenAction};
use zenoh_flow::types::{Data, NodeOutput};
use zenoh_flow::{Context, PortId, ZFError, ZFResult};

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
        traceback.format().map_or_else(|_| "".to_string(), |s| s)
    } else {
        "".to_string()
    };

    let err_str = format!("Error: {:?}\nTraceback: {:?}", py_err, tb);
    ZFError::InvalidData(err_str)
}

/// Converts rust `Context` into a `PyAny` to be passed to Python.
pub fn from_context_to_pyany<'a>(
    ctx: &'_ Context,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyAny> {
    zf_types_module
        .getattr("Context")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .call1((ctx.mode,))
        .map_err(|e| from_pyerr_to_zferr(e, py))
}

/// Converts Python `PyAny` into a rust `Context`.
pub fn from_pyany_to_context<'a>(from: &'a PyAny, py: &'a Python) -> ZFResult<Context> {
    let mode: usize = from
        .call_method0("get_mode")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .extract()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?;

    Ok(Context { mode })
}

/// Converts rust `OutputDescriptor` into a `PyAny` to be passed to Python.
pub fn from_output_descriptor_to_pyany<'a>(
    from: &'_ OutputDescriptor,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyAny> {
    zf_types_module
        .getattr("FromDescriptor")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .call1((
            PyString::new(*py, &from.node),
            PyString::new(*py, &from.output),
        ))
        .map_err(|e| from_pyerr_to_zferr(e, py))
}
/// Converts Python `PyAny` into a rust `OutputDescriptor`.
pub fn from_pyany_to_output_descritptor<'a>(
    from: &'a PyAny,
    py: &'a Python,
) -> ZFResult<OutputDescriptor> {
    let node = from
        .getattr("node")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .cast_as::<PyString>()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
        .to_str()
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .to_string()
        .into();

    let output = from
        .getattr("output")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .cast_as::<PyString>()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
        .to_str()
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .to_string()
        .into();

    Ok(OutputDescriptor { node, output })
}

/// Converts rust `InputDescriptor` into a `PyAny` to be passed to Python.
pub fn from_input_descriptor_to_pyany<'a>(
    from: &'_ InputDescriptor,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyAny> {
    zf_types_module
        .getattr("FromDescriptor")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .call1((
            PyString::new(*py, &from.node),
            PyString::new(*py, &from.input),
        ))
        .map_err(|e| from_pyerr_to_zferr(e, py))
}
/// Converts Python `PyAny` into a rust `InputDescriptor`.
pub fn from_pyany_to_input_descritptor<'a>(
    from: &'a PyAny,
    py: &'a Python,
) -> ZFResult<InputDescriptor> {
    let node = from
        .getattr("node")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .cast_as::<PyString>()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
        .to_str()
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .to_string()
        .into();

    let input = from
        .getattr("input")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .cast_as::<PyString>()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
        .to_str()
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .to_string()
        .into();

    Ok(InputDescriptor { node, input })
}

/// Converts rust `E2EDeadlineMiss` into a `PyAny` to be passed to Python.
pub fn from_e2e_deadline_miss_to_pyany<'a>(
    from: &'_ E2EDeadlineMiss,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyAny> {
    zf_types_module
        .getattr("E2EDeadlineMiss")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .call1((
            from_output_descriptor_to_pyany(&from.from, py, zf_types_module)?,
            from_input_descriptor_to_pyany(&from.to, py, zf_types_module)?,
            from.start.get_time().as_u64(),
            from.end.get_time().as_u64(),
        ))
        .map_err(|e| from_pyerr_to_zferr(e, py))
}

/// Converts Python `PyAny` into a rust `E2EDeadlineMiss`.
///
///  # Errors
/// This function is not implemented it will return an error variant.
pub fn from_pyany_to_e2e_deadline_miss<'a>(
    from: &'a PyAny,
    py: &'a Python,
) -> ZFResult<E2EDeadlineMiss> {
    let _r_from = from_pyany_to_output_descritptor(
        from.getattr("frm")
            .map_err(|e| from_pyerr_to_zferr(e, py))?,
        py,
    )?;
    let _r_to = from_pyany_to_input_descritptor(
        from.getattr("to").map_err(|e| from_pyerr_to_zferr(e, py))?,
        py,
    )?;

    Err(ZFError::Unimplemented)
    // let node = py_ctx.getattr("node")
    //         .map_err(|e| from_pyerr_to_zferr(e, py))?
    //         .cast_as::<PyString>()
    //         .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
    //         .to_str()
    //         .map_err(|e| from_pyerr_to_zferr(e, py))?
    //         .to_string()
    //         .into();

    //     let input = py_ctx.getattr("input")
    //         .map_err(|e| from_pyerr_to_zferr(e, py))?
    //         .cast_as::<PyString>()
    //         .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
    //         .to_str()
    //         .map_err(|e| from_pyerr_to_zferr(e, py))?
    //         .to_string()
    //         .into();

    //     Ok(E2EDeadlineMiss{node, input})
}

/// Converts rust `LocalDeadlineMiss` into a `PyAny` to be passed to Python.
pub fn from_local_deadline_miss_to_pyany<'a>(
    from: &'_ LocalDeadlineMiss,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyAny> {
    zf_types_module
        .getattr("LocalDeadlineMiss")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .call1((from.deadline.as_micros(), from.elapsed.as_micros()))
        .map_err(|e| from_pyerr_to_zferr(e, py))
}

/// Converts Python `PyAny` into a rust `:LocalDeadlineMiss`.
///
///  # Errors
/// This function is not implemented it will return an error variant.
pub fn from_pyany_to_local_deadline_miss<'a>(
    from: &'a PyAny,
    py: &'a Python,
) -> ZFResult<LocalDeadlineMiss> {
    let _deadline = Duration::from_micros(
        from.getattr("deadline")
            .map_err(|e| from_pyerr_to_zferr(e, py))?
            .extract()
            .map_err(|e| from_pyerr_to_zferr(e.into(), py))?,
    );

    let _elapsed = Duration::from_micros(
        from.getattr("elapsed")
            .map_err(|e| from_pyerr_to_zferr(e, py))?
            .extract()
            .map_err(|e| from_pyerr_to_zferr(e.into(), py))?,
    );

    // Ok(LocalDeadlineMiss{deadline, elapsed}

    Err(ZFError::Unimplemented)
}

/// Converts rust `Timestamp` into a `PyAny` to be passed to Python.
pub fn from_timestamp_to_pyany<'a>(
    from: &'_ Timestamp,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyAny> {
    zf_types_module
        .getattr("Timestamp")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .call1((
            from.get_time().as_u64(),
            PyString::new(*py, &from.get_id().to_string()),
        ))
        .map_err(|e| from_pyerr_to_zferr(e, py))
}

/// Converts Python `PyAny` into a rust `Timestamp`.
pub fn from_pyany_to_timestamp<'a>(from: &'a PyAny, py: &'a Python) -> ZFResult<Timestamp> {
    let ntp: u64 = from
        .getattr("ntp")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .extract()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?;

    let id = from
        .getattr("id")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .cast_as::<PyString>()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
        .to_str()
        .map_err(|e| from_pyerr_to_zferr(e, py))?;

    let id = Uuid::parse_str(id).map_err(|_| ZFError::DeseralizationError)?;
    Ok(Timestamp::new(NTP64(ntp), id.into()))
}

/// Converts rust `InputToken` Status into a `PyAny` to be passed to Python.
pub fn from_input_token_status_to_pyany<'a>(
    from: &'_ InputToken,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyAny> {
    let status = zf_types_module
        .getattr("TokenStatus")
        .map_err(|e| from_pyerr_to_zferr(e, py))?;

    match from {
        InputToken::Pending => status
            .call1((1u64,))
            .map_err(|e| from_pyerr_to_zferr(e, py)),
        InputToken::Ready(_) => status
            .call1((0u64,))
            .map_err(|e| from_pyerr_to_zferr(e, py)),
    }
}

/// Converts rust `TokenAction` into a `PyAny` to be passed to Python.
pub fn from_token_action_to_pyany<'a>(
    from: &'_ TokenAction,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyAny> {
    let status = zf_types_module
        .getattr("TokenAction")
        .map_err(|e| from_pyerr_to_zferr(e, py))?;

    match from {
        TokenAction::Consume => status
            .call1((1u64,))
            .map_err(|e| from_pyerr_to_zferr(e, py)),
        TokenAction::Drop => status
            .call1((0u64,))
            .map_err(|e| from_pyerr_to_zferr(e, py)),
        TokenAction::Keep => status
            .call1((2u64,))
            .map_err(|e| from_pyerr_to_zferr(e, py)),
    }
}

/// Converts Python `PyAny` into a rust `TokenAction`.
pub fn from_pyany_to_token_action<'a>(from: &'a PyAny, py: &'a Python) -> ZFResult<TokenAction> {
    let value: u64 = from
        .getattr("value")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .extract()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?;

    match value {
        0 => Ok(TokenAction::Drop),
        1 => Ok(TokenAction::Consume),
        2 => Ok(TokenAction::Keep),
        _ => Err(ZFError::InvalidData(format!(
            "Unable to convert to rust TokenAction possible value 0,1,2 got value {}",
            value
        ))),
    }
}

/// Converts rust `InputToken` into a `PyAny` to be passed to Python.
pub fn from_input_token_to_pyany<'a>(
    from: &'_ mut InputToken,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyAny> {
    let pyany = zf_types_module
        .getattr("InputToken")
        .map_err(|e| from_pyerr_to_zferr(e, py))?;

    let status = from_input_token_status_to_pyany(from, py, zf_types_module)?;

    match from {
        InputToken::Pending => pyany
            .call1((status,))
            .map_err(|e| from_pyerr_to_zferr(e, py)),
        InputToken::Ready(ref mut data_token) => pyany
            .call1((
                status,
                from_token_action_to_pyany(data_token.get_action(), py, zf_types_module)?,
                PyBytes::new(*py, &data_token.get_data_mut().try_as_bytes()?),
                from_timestamp_to_pyany(data_token.get_timestamp(), py, zf_types_module)?,
            ))
            .map_err(|e| from_pyerr_to_zferr(e, py)),
    }
}

/// Converts rust `HashMap<PortId, InputToken>` into a `PyDict` to be passed to Python.
pub fn from_input_tokens_to_pydict<'a>(
    from: &'_ mut HashMap<PortId, InputToken>,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyDict> {
    let pydict = PyDict::new(*py);

    for (k, v) in from {
        let data = from_input_token_to_pyany(v, py, zf_types_module)?;

        pydict
            .set_item(PyString::new(*py, k), data)
            .map_err(|e| from_pyerr_to_zferr(e, py))?
    }
    Ok(pydict)
}

/// Converts Python `PyDict` into a rust `HashMap<PortId, InputToken>`.
pub fn from_pydict_to_input_tokens<'a>(
    pydict: &'a PyDict,
    py: &'a Python,
) -> ZFResult<HashMap<PortId, InputToken>> {
    let mut tokens = HashMap::with_capacity(pydict.len());

    for (k, v) in pydict.iter() {
        let port_id = k
            .cast_as::<PyString>()
            .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
            .to_str()
            .map_err(|e| from_pyerr_to_zferr(e, py))?
            .to_string();

        let token = from_pyany_to_input_token(v, py)?;

        tokens.insert(port_id.into(), token);
    }

    Ok(tokens)
}

/// Converts Python `PyAny` into a rust `InputToken`.
pub fn from_pyany_to_input_token<'a>(from: &'a PyAny, py: &'a Python) -> ZFResult<InputToken> {
    let ready: bool = from
        .call_method0("is_ready")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .extract()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?;

    match ready {
        false => Ok(InputToken::Pending),
        true => {
            let action = from_pyany_to_token_action(
                from.getattr("action")
                    .map_err(|e| from_pyerr_to_zferr(e, py))?,
                py,
            )?;

            let data = Data::from_bytes(
                from.getattr("data")
                    .map_err(|e| from_pyerr_to_zferr(e, py))?
                    .cast_as::<PyBytes>()
                    .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
                    .as_bytes()
                    .to_vec(),
            );

            let timestamp = from_pyany_to_timestamp(
                from.getattr("timestamp")
                    .map_err(|e| from_pyerr_to_zferr(e, py))?,
                py,
            )?;

            let data_message = DataMessage::new(data, timestamp, vec![]);

            Ok(InputToken::Ready(DataToken::new(action, data_message)))
        }
    }
}

/// Converts rust `DataMessage` into a `PyAny` to be passed to Python.
pub fn from_data_message_to_pyany<'a>(
    from: &'_ mut DataMessage,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyAny> {
    // let pye2e = PyList::empty(*py);

    // for e in from.get_missed_end_to_end_deadlines() {
    //     pye2e
    //         .append(from_e2e_deadline_miss_to_pyany(e, py, zf_types_module)?)
    //         .map_err(|e| from_pyerr_to_zferr(e, py))?;
    // }

    let py_it = zf_types_module
        .getattr("DataMessage")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .call1((
            from_timestamp_to_pyany(from.get_timestamp(), py, zf_types_module)?,
            PyBytes::new(*py, &from.get_inner_data().try_as_bytes()?),
        ))
        .map_err(|e| from_pyerr_to_zferr(e, py))?;

    Ok(py_it)
}

/// Converts Python `PyAny` into a rust `DataMessage`.
pub fn from_pyany_to_data_message<'a>(from: &'a PyAny, py: &'a Python) -> ZFResult<DataMessage> {
    let data = from
        .getattr("data")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .cast_as::<PyBytes>()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
        .as_bytes()
        .to_vec();

    let ts = from_pyany_to_timestamp(
        from.getattr("data")
            .map_err(|e| from_pyerr_to_zferr(e, py))?,
        py,
    )?;

    // let mut e2ed = vec![];
    // let pye2ed = from.getattr("missed_end_to_end_deadlines")
    //     .map_err(|e| from_pyerr_to_zferr(e, py))?;

    // for e in pye2ed.iter() {
    //     e2ed.push(from_pyany_to_e2e_deadline_miss(e, py)?);
    // }

    Ok(DataMessage::new(Data::from_bytes(data), ts, vec![]))
}

/// Converts Python `PyAny` into a rust `DataMessage`.
pub fn from_pyany_to_run_result<'a>(
    from: &'a PyAny,
    py: &'a Python,
) -> ZFResult<HashMap<PortId, Data>> {
    let pydict = from
        .cast_as::<PyDict>()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?;

    let mut outputs = HashMap::with_capacity(pydict.len());

    for (k, v) in pydict.iter() {
        let port_id = k
            .cast_as::<PyString>()
            .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
            .to_str()
            .map_err(|e| from_pyerr_to_zferr(e, py))?
            .to_string();

        let data = Data::from_bytes(
            v.cast_as::<PyBytes>()
                .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
                .as_bytes()
                .to_vec(),
        );

        outputs.insert(port_id.into(), data);
    }

    Ok(outputs)
}

/// Converts rust `HashMap<PortId, DataMessage>` into a `PyDict` to be passed to Python.
pub fn from_inputs_to_pydict<'a>(
    from: &'_ mut HashMap<PortId, DataMessage>,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyDict> {
    let pydict = PyDict::new(*py);

    for (k, v) in from {
        let data = from_data_message_to_pyany(v, py, zf_types_module)?;

        pydict
            .set_item(PyString::new(*py, k), data)
            .map_err(|e| from_pyerr_to_zferr(e, py))?
    }
    Ok(pydict)
}

/// Converts Python `PyAny` into a rust `Data`.
pub fn from_pyany_to_data<'a>(from: &'a PyAny, py: &'a Python) -> ZFResult<Data> {
    Ok(Data::from_bytes(
        from.cast_as::<PyBytes>()
            .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
            .as_bytes()
            .to_vec(),
    ))
}

/// Converts rust `DataMessage` into a `PyAny` to be passed to Python.
pub fn from_data_to_pybytes<'a>(from: &'_ mut Data, py: &'a Python) -> ZFResult<&'a PyBytes> {
    Ok(PyBytes::new(*py, &from.try_as_bytes()?))
}

/// Converts rust `HashMap<PortId, Data>` into a `PyDict` to be passed to Python.
pub fn from_outputs_to_pydict<'a>(
    from: &'_ mut HashMap<PortId, Data>,
    py: &'a Python,
) -> ZFResult<&'a PyDict> {
    let pydict = PyDict::new(*py);

    for (k, v) in from {
        let data = from_data_to_pybytes(v, py)?;

        pydict
            .set_item(PyString::new(*py, k), data)
            .map_err(|e| from_pyerr_to_zferr(e, py))?
    }
    Ok(pydict)
}

/// Converts Python `PyDict` into a rust `HashMap<PortId, NodeOutput>`.
pub fn from_pyany_to_or_result<'a>(
    from: &'a PyAny,
    py: &'a Python,
) -> ZFResult<HashMap<PortId, NodeOutput>> {
    let pydict = from
        .cast_as::<PyDict>()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?;

    let mut outputs = HashMap::with_capacity(pydict.len());

    for (k, v) in pydict.iter() {
        let port_id = k
            .cast_as::<PyString>()
            .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
            .to_str()
            .map_err(|e| from_pyerr_to_zferr(e, py))?
            .to_string();

        let data = NodeOutput::Data(Data::from_bytes(
            v.cast_as::<PyBytes>()
                .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
                .as_bytes()
                .to_vec(),
        ));

        outputs.insert(port_id.into(), data);
    }

    Ok(outputs)
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
