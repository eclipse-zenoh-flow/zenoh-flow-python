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

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyList, PyString};
use std::collections::HashMap;
use std::convert::TryInto;
use std::convert::{From, TryFrom};
use std::time::Duration;
use zenoh_flow::{Data, PortId, ZFError, ZFResult};

pub mod utils;

pub fn from_pyerr_to_zferr(py_err: pyo3::PyErr, py: &Python) -> ZFError {
    let tb = py_err
        .traceback(*py)
        .expect("This error should have a traceback");
    let err_str = format!("Error: {:?}\nTraceback: {:?}", py_err, tb.format());
    ZFError::InvalidData(err_str)
}

/// Converts rust `zenoh_flow::Context` into a `PyAny` to be passed to Python.
pub fn from_context_to_pyany<'a>(
    ctx: &'_ zenoh_flow::Context,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyAny> {
    zf_types_module
        .getattr("Context")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .call1((ctx.mode,))
        .map_err(|e| from_pyerr_to_zferr(e, py))
}

/// Converts Python `PyAny` into a rust `zenoh_flow::Context`.
pub fn from_pyany_to_context<'a>(from: &'a PyAny, py: &'a Python) -> ZFResult<zenoh_flow::Context> {
    let mode: usize = from
        .call_method0("get_mode")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .extract()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?;

    Ok(zenoh_flow::Context { mode })
}

/// Converts rust `zenoh_flow::model::OutputDescriptor` into a `PyAny` to be passed to Python.
pub fn from_output_descriptor_to_pyany<'a>(
    from: &'_ zenoh_flow::model::OutputDescriptor,
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
/// Converts Python `PyAny` into a rust `zenoh_flow::model::OutputDescriptor`.
pub fn from_pyany_to_output_descritptor<'a>(
    from: &'a PyAny,
    py: &'a Python,
) -> ZFResult<zenoh_flow::model::OutputDescriptor> {
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

    Ok(zenoh_flow::model::OutputDescriptor { node, output })
}

/// Converts rust `zenoh_flow::model::InputDescriptor` into a `PyAny` to be passed to Python.
pub fn from_input_descriptor_to_pyany<'a>(
    from: &'_ zenoh_flow::model::InputDescriptor,
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
/// Converts Python `PyAny` into a rust `zenoh_flow::model::InputDescriptor`.
pub fn from_pyany_to_input_descritptor<'a>(
    from: &'a PyAny,
    py: &'a Python,
) -> ZFResult<zenoh_flow::model::InputDescriptor> {
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

    Ok(zenoh_flow::model::InputDescriptor { node, input })
}

/// Converts rust `zenoh_flow::runtime::deadline::E2EDeadlineMiss` into a `PyAny` to be passed to Python.
pub fn from_e2e_deadline_miss_to_pyany<'a>(
    from: &'_ zenoh_flow::runtime::deadline::E2EDeadlineMiss,
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

/// Converts Python `PyAny` into a rust `zenoh_flow::runtime::deadline::E2EDeadlineMiss`.
pub fn from_pyany_to_e2e_deadline_miss<'a>(
    from: &'a PyAny,
    py: &'a Python,
) -> ZFResult<zenoh_flow::runtime::deadline::E2EDeadlineMiss> {
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

    //     Ok(zenoh_flow::runtime::deadline::E2EDeadlineMiss{node, input})
}

/// Converts rust `zenoh_flow::runtime::deadline::LocalDeadlineMiss` into a `PyAny` to be passed to Python.
pub fn from_local_deadline_miss_to_pyany<'a>(
    from: &'_ zenoh_flow::runtime::deadline::LocalDeadlineMiss,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyAny> {
    zf_types_module
        .getattr("LocalDeadlineMiss")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .call1((from.deadline.as_micros(), from.elapsed.as_micros()))
        .map_err(|e| from_pyerr_to_zferr(e, py))
}

/// Converts Python `PyAny` into a rust `zenoh_flow::runtime::deadline::LocalDeadlineMiss`.
pub fn from_pyany_to_local_deadline_miss<'a>(
    from: &'a PyAny,
    py: &'a Python,
) -> ZFResult<zenoh_flow::runtime::deadline::LocalDeadlineMiss> {
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

    // Ok(zenoh_flow::runtime::deadline::LocalDeadlineMiss{deadline, elapsed}

    Err(ZFError::Unimplemented)
}

/// Converts rust `uhlc::Timestamp` into a `PyAny` to be passed to Python.
pub fn from_timestamp_to_pyany<'a>(
    from: &'_ uhlc::Timestamp,
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

/// Converts Python `PyAny` into a rust `uhlc::Timestamp`.
pub fn from_pyany_to_timestamp<'a>(from: &'a PyAny, py: &'a Python) -> ZFResult<uhlc::Timestamp> {
    let _ntp: u64 = from
        .getattr("ntp")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .extract()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?;

    let _id = from
        .getattr("id")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .cast_as::<PyString>()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
        .to_str()
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .to_string();

    // Ok(zenoh_flow::runtime::deadline::LocalDeadlineMiss{deadline, elapsed}

    // Ok(uhlc::Timestamp::new(ntp, id))
    Err(ZFError::Unimplemented)
}

/// Converts rust `zenoh_flow::runtime::token::InputToken` Status into a `PyAny` to be passed to Python.
pub fn from_input_token_status_to_pyany<'a>(
    from: &'_ zenoh_flow::runtime::token::InputToken,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyAny> {
    let status = zf_types_module
        .getattr("TokenStatus")
        .map_err(|e| from_pyerr_to_zferr(e, py))?;

    match from {
        zenoh_flow::runtime::token::InputToken::Pending => status
            .call1((1u64,))
            .map_err(|e| from_pyerr_to_zferr(e, py)),
        zenoh_flow::runtime::token::InputToken::Ready(_) => status
            .call1((0u64,))
            .map_err(|e| from_pyerr_to_zferr(e, py)),
    }
}

/// Converts rust `zenoh_flow::runtime::token::TokenAction` into a `PyAny` to be passed to Python.
pub fn from_token_action_to_pyany<'a>(
    from: &'_ zenoh_flow::runtime::token::TokenAction,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyAny> {
    let status = zf_types_module
        .getattr("TokenAction")
        .map_err(|e| from_pyerr_to_zferr(e, py))?;

    match from {
        zenoh_flow::runtime::token::TokenAction::Consume => status
            .call1((1u64,))
            .map_err(|e| from_pyerr_to_zferr(e, py)),
        zenoh_flow::runtime::token::TokenAction::Drop => status
            .call1((0u64,))
            .map_err(|e| from_pyerr_to_zferr(e, py)),
        zenoh_flow::runtime::token::TokenAction::Keep => status
            .call1((2u64,))
            .map_err(|e| from_pyerr_to_zferr(e, py)),
    }
}

/// Converts Python `PyAny` into a rust `zenoh_flow::runtime::token::TokenAction`.
pub fn from_pyany_to_token_action<'a>(
    from: &'a PyAny,
    py: &'a Python,
) -> ZFResult<zenoh_flow::runtime::token::TokenAction> {
    let value: u64 = from
        .getattr("value")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .extract()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?;

    match value {
        0 => Ok(zenoh_flow::runtime::token::TokenAction::Drop),
        1 => Ok(zenoh_flow::runtime::token::TokenAction::Consume),
        2 => Ok(zenoh_flow::runtime::token::TokenAction::Keep),
        _ => Err(ZFError::InvalidData(format!(
            "Unable to convert to rust TokenAction possible value 0,1,2 got value {}",
            value
        ))),
    }
}

/// Converts rust `zenoh_flow::runtime::token::InputToken` into a `PyAny` to be passed to Python.
pub fn from_input_token_to_pyany<'a>(
    from: &'_ mut zenoh_flow::runtime::token::InputToken,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyAny> {
    let py_it = zf_types_module
        .getattr("InputToken")
        .map_err(|e| from_pyerr_to_zferr(e, py))?;

    let status = from_input_token_status_to_pyany(from, py, zf_types_module)?;

    match from {
        zenoh_flow::runtime::token::InputToken::Pending => {
            py_it
                .call1((status,))
                .map_err(|e| from_pyerr_to_zferr(e, py))?;

            Ok(py_it)
        }
        zenoh_flow::runtime::token::InputToken::Ready(ref mut data_token) => {
            py_it
                .call1((
                    status,
                    from_token_action_to_pyany(data_token.get_action(), py, zf_types_module)?,
                    PyBytes::new(*py, &data_token.get_data_mut().try_as_bytes()?),
                ))
                .map_err(|e| from_pyerr_to_zferr(e, py))?;

            Ok(py_it)
        }
    }
}

/// Converts Python `PyAny` into a rust `zenoh_flow::runtime::token::InputToken`.
pub fn from_pyany_to_input_token<'a>(
    from: &'a PyAny,
    py: &'a Python,
) -> ZFResult<zenoh_flow::runtime::token::InputToken> {
    let ready: bool = from
        .call_method0("is_ready")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .extract()
        .map_err(|e| from_pyerr_to_zferr(e.into(), py))?;

    match ready {
        false => Ok(zenoh_flow::runtime::token::InputToken::Pending),
        true => {
            let _action = from_pyany_to_token_action(
                from.getattr("action")
                    .map_err(|e| from_pyerr_to_zferr(e, py))?,
                py,
            )?;

            let _data = from
                .getattr("data")
                .map_err(|e| from_pyerr_to_zferr(e, py))?
                .cast_as::<PyBytes>()
                .map_err(|e| from_pyerr_to_zferr(e.into(), py))?
                .as_bytes()
                .to_vec();

            Err(ZFError::Unimplemented)
        }
    }
}

/// Converts rust `zenoh_flow::runtime::message::DataMessage` into a `PyAny` to be passed to Python.
pub fn from_data_message_to_pyany<'a>(
    from: &'_ mut zenoh_flow::runtime::message::DataMessage,
    py: &'a Python,
    zf_types_module: &'a PyModule,
) -> ZFResult<&'a PyAny> {
    let pye2e = PyList::empty(*py);

    for e in from.get_missed_end_to_end_deadlines() {
        pye2e
            .append(from_e2e_deadline_miss_to_pyany(e, py, zf_types_module)?)
            .map_err(|e| from_pyerr_to_zferr(e, py))?;
    }

    let py_it = zf_types_module
        .getattr("DataMessage")
        .map_err(|e| from_pyerr_to_zferr(e, py))?
        .call1((
            from_timestamp_to_pyany(from.get_timestamp(), py, zf_types_module)?,
            pye2e,
            PyBytes::new(*py, &from.get_inner_data().try_as_bytes()?),
        ))
        .map_err(|e| from_pyerr_to_zferr(e, py))?;

    Ok(py_it)
}

/// Converts Python `PyAny` into a rust `zenoh_flow::runtime::message::DataMessage`.
pub fn from_pyany_to_data_message<'a>(
    from: &'a PyAny,
    py: &'a Python,
) -> ZFResult<zenoh_flow::runtime::message::DataMessage> {
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

    Ok(zenoh_flow::runtime::message::DataMessage::new(
        zenoh_flow::types::Data::from_bytes(data),
        ts,
        vec![],
    ))
}

/// Converts Python `PyAny` into a rust `zenoh_flow::runtime::message::DataMessage`.
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

// --------------------------------------------------- OLD --------------------

// /// A Zenoh Flow Data Message.
// /// It contains:
// /// `data` as array of bytes.
// /// `ts` an uHLC timestamp associated with the data.
// /// `missed_end_to_end_deadlines` list of `E2EDeadlineMiss`
// #[pyclass]
// #[derive(Clone)]
// pub struct DataMessage {
//     pub(crate) data: Vec<u8>,
//     pub(crate) ts: uhlc::Timestamp,
//     pub(crate) missed_end_to_end_deadlines: Vec<E2EDeadlineMiss>,
// }

// #[pymethods]
// impl DataMessage {
//     /// Gets the timestamp from the :class:`Data Message`
//     ///
//     /// :rtype: str
//     #[getter]
//     fn timestamp(&self) -> String {
//         self.ts.to_string()
//     }

//     /// Gets the data from the :class:`Data Message`
//     ///
//     /// :rtype: bytes
//     #[getter]
//     fn data(&self) -> &[u8] {
//         &self.data
//     }

//     /// Gets the missed end to end deadlines from the :class:`Data Message`
//     ///
//     /// :rtype: list[`E2EDeadlineMiss`]
//     #[getter]
//     fn missed_end_to_end_deadlines(&self) -> Vec<E2EDeadlineMiss> {
//         self.missed_end_to_end_deadlines.clone()
//     }

//     fn __str__(&self) -> PyResult<String> {
//         Ok(format!("Timestamp {:?} - Data: {:?}", self.ts, self.data))
//     }

//     fn __repr__(&self) -> PyResult<String> {
//         self.__str__()
//     }
// }

// impl TryFrom<zenoh_flow::DataMessage> for DataMessage {
//     type Error = ZFError;
//     fn try_from(mut msg: zenoh_flow::DataMessage) -> Result<Self, Self::Error> {
//         let missed_end_to_end_deadlines: Vec<E2EDeadlineMiss> = msg
//             .get_missed_end_to_end_deadlines()
//             .iter()
//             .map(|e2e_deadline| e2e_deadline.into())
//             .collect();

//         Ok(Self {
//             ts: *msg.get_timestamp(),
//             data: msg.get_inner_data().try_as_bytes()?.to_vec(),
//             missed_end_to_end_deadlines,
//         })
//     }
// }

// impl TryFrom<&mut zenoh_flow::DataMessage> for DataMessage {
//     type Error = ZFError;
//     fn try_from(msg: &mut zenoh_flow::DataMessage) -> Result<Self, Self::Error> {
//         let missed_end_to_end_deadlines: Vec<E2EDeadlineMiss> = msg
//             .get_missed_end_to_end_deadlines()
//             .iter()
//             .map(|e2e_deadline| e2e_deadline.into())
//             .collect();

//         Ok(Self {
//             ts: *msg.get_timestamp(),
//             data: msg.get_inner_data().try_as_bytes()?.to_vec(),
//             missed_end_to_end_deadlines,
//         })
//     }
// }

// /// The inputs received in the Operator run function.
// #[pyclass]
// #[derive(Clone)]
// pub struct Inputs {
//     pub(crate) inputs: HashMap<String, DataMessage>,
// }

// #[pymethods]
// impl Inputs {
//     /// Gets the :class:`DataMessage` from the :class:`Inputs`
//     ///
//     /// :param id: The ID of the input port.
//     /// :type id: str
//     ///
//     /// :rtype: :class:`DataMessage`
//     fn get(&self, id: String) -> Option<DataMessage> {
//         self.inputs.get(&id).cloned()
//     }

//     fn __str__(&self) -> PyResult<String> {
//         Ok(format!("Total data {}", self.inputs.len()))
//     }

//     fn __repr__(&self) -> PyResult<String> {
//         self.__str__()
//     }
// }

// impl TryFrom<&mut HashMap<zenoh_flow::PortId, zenoh_flow::DataMessage>> for Inputs {
//     type Error = ZFError;
//     fn try_from(
//         rust_inputs: &mut HashMap<zenoh_flow::PortId, zenoh_flow::DataMessage>,
//     ) -> Result<Self, Self::Error> {
//         let mut inputs = HashMap::new();
//         for (k, v) in rust_inputs {
//             let port_id = k.to_string();
//             let data = DataMessage::try_from(v)?;
//             inputs.insert(port_id, data);
//         }
//         Ok(Self { inputs })
//     }
// }

// /// Zenoh Flow outputs, passed to the operator output rules
// #[pyclass]
// #[derive(Clone)]
// pub struct Outputs {
//     pub(crate) outputs: HashMap<String, Vec<u8>>,
// }

// #[pymethods]
// impl Outputs {
//     #[new]
//     fn new() -> Self {
//         Self {
//             outputs: HashMap::new(),
//         }
//     }
//     /// Adds a value to the :class:`Outputs`
//     ///
//     /// :param id: the ID of the output port
//     /// :type id: str
//     /// :param data: The data
//     /// :type id: bytes
//     ///
//     fn put(&mut self, id: String, data: Vec<u8>) {
//         self.outputs.insert(id, data);
//     }

//     /// Gets the data from the :class:`Outputs
//     /// `
//     /// :param id: The ID of the output port.
//     /// :type id: str
//     ///
//     /// :rtype: bytes
//     fn get(&self, id: String) -> Option<Vec<u8>> {
//         self.outputs.get(&id).cloned()
//     }

//     fn __str__(&self) -> PyResult<String> {
//         Ok(format!("Total data {}", self.outputs.len()))
//     }

//     fn __repr__(&self) -> PyResult<String> {
//         self.__str__()
//     }
// }

// impl IntoIterator for Outputs {
//     type Item = (String, Vec<u8>);
//     type IntoIter = std::collections::hash_map::IntoIter<String, Vec<u8>>;

//     fn into_iter(self) -> Self::IntoIter {
//         self.outputs.into_iter()
//     }
// }

// impl TryFrom<HashMap<zenoh_flow::PortId, zenoh_flow::Data>> for Outputs {
//     type Error = ZFError;
//     fn try_from(
//         rust_inputs: HashMap<zenoh_flow::PortId, zenoh_flow::Data>,
//     ) -> Result<Self, Self::Error> {
//         let mut outputs = HashMap::new();
//         for (k, v) in rust_inputs {
//             let port_id = k.to_string();
//             let data = v.try_as_bytes()?.to_vec();
//             outputs.insert(port_id, data);
//         }
//         Ok(Self { outputs })
//     }
// }

// impl TryFrom<(Py<PyDict>, Python<'_>)> for Outputs {
//     type Error = ZFError;
//     fn try_from(dict: (Py<PyDict>, Python<'_>)) -> Result<Self, Self::Error> {
//         let (dict, py) = dict;
//         let mut outputs = HashMap::new();
//         for (k, v) in dict.as_ref(py).into_iter() {
//             let port_id = k.to_string();
//             let data: Vec<u8> = v
//                 .extract()
//                 .map_err(|e| ZFError::InvalidData(e.to_string()))?;
//             outputs.insert(port_id, data);
//         }
//         Ok(Self { outputs })
//     }
// }

// impl TryInto<HashMap<zenoh_flow::PortId, zenoh_flow::Data>> for Outputs {
//     type Error = ZFError;
//     fn try_into(self) -> Result<HashMap<zenoh_flow::PortId, zenoh_flow::Data>, Self::Error> {
//         let mut outputs = HashMap::new();
//         for (k, v) in self.outputs {
//             let data = zenoh_flow::Data::from_bytes(v);
//             outputs.insert(k.into(), data);
//         }
//         Ok(outputs)
//     }
// }

// /// A Zenoh Flow Input Token
// #[pyclass]
// #[derive(Clone, Debug)]
// pub struct InputToken {
//     pub(crate) token: zenoh_flow::InputToken,
// }

// #[pymethods]
// impl InputToken {
//     /// Sets the token to be dropped.
//     pub fn set_action_drop(&mut self) {
//         self.token.set_action_drop()
//     }

//     /// Sets the token to be kept for next iteration.
//     pub fn set_action_keep(&mut self) {
//         self.token.set_action_keep()
//     }

//     /// Sets the token to be consumed in the current iteration (default).
//     pub fn set_action_consume(&mut self) {
//         self.token.set_action_consume()
//     }

//     /// Gets the timestamp from the :class:`Token`.
//     ///
//     /// :rtype: str
//     pub fn get_timestamp(&self) -> String {
//         match &self.token {
//             zenoh_flow::InputToken::Ready(ref r) => r.get_timestamp().to_string(),
//             _ => String::from(""),
//         }
//     }

//     /// Gets the data from the :class:`Token`
//     ///
//     /// :rtype: bytes
//     pub fn get_data(&mut self) -> PyResult<Vec<u8>> {
//         match &mut self.token {
//             zenoh_flow::InputToken::Ready(ref mut r) => {
//                 let data = r.get_data_mut();
//                 Ok(data
//                     .try_as_bytes()
//                     .map_err(|_| {
//                         pyo3::exceptions::PyValueError::new_err("Unable to get data from token")
//                     })?
//                     .to_vec())
//             }
//             _ => Err(pyo3::exceptions::PyValueError::new_err(
//                 "Pending Token has no data",
//             )),
//         }
//     }

//     /// Gets the action from the :class:`Token`
//     ///
//     /// :rtype: str
//     pub fn get_action(&self) -> String {
//         match &self.token {
//             zenoh_flow::InputToken::Ready(ref r) => r.get_action().to_string(),
//             _ => String::from("Pending"),
//         }
//     }

//     /// Checks if the :class:`Token` is ready.
//     /// i.e. has Data.
//     ///
//     /// :rtype: bool
//     pub fn is_ready(&self) -> bool {
//         matches!(&self.token, zenoh_flow::InputToken::Ready(_))
//     }

//     /// Checks if the :class:`Token` is pending.
//     /// i.e. has no data.
//     ///
//     /// :rtype: bool
//     pub fn is_pending(&self) -> bool {
//         matches!(&self.token, zenoh_flow::InputToken::Pending)
//     }
// }

// impl From<zenoh_flow::InputToken> for InputToken {
//     fn from(token: zenoh_flow::InputToken) -> Self {
//         Self { token }
//     }
// }

// impl From<InputToken> for zenoh_flow::InputToken {
//     fn from(val: InputToken) -> Self {
//         val.token
//     }
// }

// /// A set of :class:`InputTokens`
// #[pyclass]
// #[derive(Clone, Debug)]
// pub struct InputTokens {
//     pub(crate) tokens: HashMap<String, InputToken>,
// }

// #[pymethods]
// impl InputTokens {
//     /// Gets the :class:`InputToken` for the given port ID.
//     ///
//     /// :param port_id: The input port ID.
//     /// :type port_id: str
//     ///
//     /// :rtype: :class:`InputToken`
//     pub fn get(&mut self, port_id: String) -> PyResult<InputToken> {
//         match self.tokens.get(&port_id) {
//             Some(t) => Ok(t.clone()),
//             None => Err(pyo3::exceptions::PyValueError::new_err(format!(
//                 "No tokens found for the given port_id {}",
//                 port_id
//             ))),
//         }
//     }
// }

// // impl From<zenoh_flow::InputTokens> for Tokens {
// //     fn from(tokens: zenoh_flow::InputTokens) -> Self {
// //         Self { tokens }
// //     }
// // }

// impl From<HashMap<zenoh_flow::PortId, zenoh_flow::InputToken>> for InputTokens {
//     fn from(rust_tokens: HashMap<zenoh_flow::PortId, zenoh_flow::InputToken>) -> Self {
//         Self {
//             tokens: rust_tokens
//                 .into_iter()
//                 .map(|(id, token)| (id.to_string(), InputToken::from(token)))
//                 .collect(),
//         }
//     }
// }

// impl From<InputTokens> for HashMap<zenoh_flow::PortId, zenoh_flow::InputToken> {
//     fn from(val: InputTokens) -> Self {
//         val.tokens
//             .into_iter()
//             .map(|(id, token)| (id.into(), token.into()))
//             .collect()
//     }
// }

// /// A Zenoh Flow local deadline miss.
// /// A structure containing all the information regarding a missed, local, deadline.
// /// A local deadline is represented by a maximum time between receiving the
// /// data at the Input Rules and providing a result to the Output Rule.
// /// This means that if the Run function takes more that the deadline
// /// the Output Rule will be notified by the means of this
// /// `LocalDeadlineMiss`.
// #[pyclass]
// #[derive(Clone)]
// pub struct LocalDeadlineMiss {
//     pub(crate) deadline: u128,
//     pub(crate) elapsed: u128,
// }

// #[pymethods]
// impl LocalDeadlineMiss {
//     /// Gets the deadline.
//     ///
//     /// :rtype: int
//     #[getter]
//     fn deadline(&self) -> u128 {
//         self.deadline
//     }

//     /// Gets the elapsed time.
//     ///
//     /// :rtype: int
//     #[getter]
//     fn elapsed(&self) -> u128 {
//         self.elapsed
//     }
// }

// impl From<zenoh_flow::LocalDeadlineMiss> for LocalDeadlineMiss {
//     fn from(deadline_miss: zenoh_flow::LocalDeadlineMiss) -> Self {
//         Self {
//             deadline: deadline_miss.deadline.as_micros(),
//             elapsed: deadline_miss.elapsed.as_micros(),
//         }
//     }
// }

// impl From<Option<zenoh_flow::LocalDeadlineMiss>> for LocalDeadlineMiss {
//     fn from(deadline_miss: Option<zenoh_flow::LocalDeadlineMiss>) -> Self {
//         match deadline_miss {
//             Some(dl_miss) => Self {
//                 deadline: dl_miss.deadline.as_micros(),
//                 elapsed: dl_miss.elapsed.as_micros(),
//             },
//             None => Self {
//                 deadline: 0,
//                 elapsed: 0,
//             },
//         }
//     }
// }

// /// The descriptor on where an E2E Deadline starts.
// ///
// #[pyclass]
// #[derive(Clone)]
// pub struct FromDescriptor {
//     pub node: String,
//     pub output: String,
// }

// #[pymethods]
// impl FromDescriptor {
//     /// Gets the node ID from :class:`FromDescriptor`
//     ///
//     /// :rtype: str
//     #[getter]
//     fn node(&self) -> &str {
//         &self.node
//     }

//     /// Gets the port ID from :class:`FromDescriptor`
//     ///
//     /// :rtype: str
//     #[getter]
//     fn output(&self) -> &str {
//         &self.output
//     }
// }

// /// The descriptor on where a E2E Deadline ends.
// ///
// #[pyclass]
// #[derive(Clone)]
// pub struct ToDescriptor {
//     pub node: String,
//     pub input: String,
// }

// #[pymethods]
// impl ToDescriptor {
//     /// Gets the node ID from :class:`ToDescriptor`
//     ///
//     /// :rtype: str
//     #[getter]
//     fn node(&self) -> &str {
//         &self.node
//     }

//     /// Gets the port ID from :class:`ToDescriptor`
//     ///
//     /// :rtype: str
//     #[getter]
//     fn input(&self) -> &str {
//         &self.input
//     }
// }

// /// A End to End Deadline.
// /// A deadline can apply for a whole graph or for a subpart of it.
// #[pyclass]
// #[derive(Clone)]
// pub struct E2EDeadlineMiss {
//     pub from: FromDescriptor,
//     pub to: ToDescriptor,
//     pub start: u64,
//     pub end: u64,
// }

// #[pymethods]
// impl E2EDeadlineMiss {
//     /// Gets from where the deadline starts.
//     ///
//     /// :rtype: :class:`FromDescriptor`
//     #[getter]
//     fn from(&self) -> FromDescriptor {
//         self.from.clone()
//     }

//     /// Gets where the deadline ends.
//     ///
//     /// :rtype: :class:`ToDescriptor`
//     #[getter]
//     fn to(&self) -> ToDescriptor {
//         self.to.clone()
//     }

//     /// Gets the start time of the deadline.
//     /// :rtype: int
//     #[getter]
//     fn start(&self) -> u64 {
//         self.start
//     }
//     /// Gets the end time of the deadline.
//     ///
//     /// :rtype: int
//     #[getter]
//     fn end(&self) -> u64 {
//         self.end
//     }
// }

// impl From<&zenoh_flow::runtime::deadline::E2EDeadlineMiss> for E2EDeadlineMiss {
//     fn from(e2d_deadline_miss: &zenoh_flow::runtime::deadline::E2EDeadlineMiss) -> Self {
//         let to = ToDescriptor {
//             node: e2d_deadline_miss.to.node.to_string(),
//             input: e2d_deadline_miss.to.input.to_string(),
//         };
//         let from = FromDescriptor {
//             node: e2d_deadline_miss.from.node.to_string(),
//             output: e2d_deadline_miss.from.output.to_string(),
//         };

//         Self {
//             from,
//             to,
//             start: e2d_deadline_miss.start.get_time().as_u64(),
//             end: e2d_deadline_miss.end.get_time().as_u64(),
//         }
//     }
// }
