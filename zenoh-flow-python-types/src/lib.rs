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
use pyo3::types::PyDict;
use std::collections::HashMap;
use std::convert::TryInto;
use std::convert::{From, TryFrom};
use zenoh_flow::ZFError;

pub mod utils;

pub fn from_pyerr_to_zferr(py_err: pyo3::PyErr, py: &pyo3::Python<'_>) -> ZFError {
    let tb = if let Some(traceback) = py_err.traceback(*py) {
        traceback.format().map_or_else(|_| "".to_string(), |s| s)
    } else {
        "".to_string()
    };

    let err_str = format!("Error: {:?}\nTraceback: {:?}", py_err, tb);
    ZFError::InvalidData(err_str)
}

/// A Zenoh Flow context.
/// Zenoh Flow context contains a `mode` that represent
/// the current execution mode for the operator.
#[pyclass]
pub struct Context {
    pub(crate) mode: usize,
}

#[pymethods]
impl Context {
    /// Gets the mode from the :class:`Context`
    ///
    /// :rtype: int
    #[getter]
    fn mode(&self) -> usize {
        self.mode
    }
}

impl From<&mut zenoh_flow::Context> for Context {
    fn from(ctx: &mut zenoh_flow::Context) -> Self {
        Self { mode: ctx.mode }
    }
}

/// A Zenoh Flow Data Message.
/// It contains:
/// `data` as array of bytes.
/// `ts` an uHLC timestamp associated with the data.
/// `missed_end_to_end_deadlines` list of `E2EDeadlineMiss`
#[pyclass]
#[derive(Clone)]
pub struct DataMessage {
    pub(crate) data: Vec<u8>,
    pub(crate) ts: uhlc::Timestamp,
    pub(crate) missed_end_to_end_deadlines: Vec<E2EDeadlineMiss>,
}

#[pymethods]
impl DataMessage {
    /// Gets the timestamp from the :class:`Data Message`
    ///
    /// :rtype: str
    #[getter]
    fn timestamp(&self) -> String {
        self.ts.to_string()
    }

    /// Gets the data from the :class:`Data Message`
    ///
    /// :rtype: bytes
    #[getter]
    fn data(&self) -> &[u8] {
        &self.data
    }

    /// Gets the missed end to end deadlines from the :class:`Data Message`
    ///
    /// :rtype: list[`E2EDeadlineMiss`]
    #[getter]
    fn missed_end_to_end_deadlines(&self) -> Vec<E2EDeadlineMiss> {
        self.missed_end_to_end_deadlines.clone()
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Timestamp {:?} - Data: {:?}", self.ts, self.data))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}

impl TryFrom<zenoh_flow::DataMessage> for DataMessage {
    type Error = ZFError;
    fn try_from(mut msg: zenoh_flow::DataMessage) -> Result<Self, Self::Error> {
        let missed_end_to_end_deadlines: Vec<E2EDeadlineMiss> = msg
            .get_missed_end_to_end_deadlines()
            .iter()
            .map(|e2e_deadline| e2e_deadline.into())
            .collect();

        Ok(Self {
            ts: *msg.get_timestamp(),
            data: msg.get_inner_data().try_as_bytes()?.to_vec(),
            missed_end_to_end_deadlines,
        })
    }
}

impl TryFrom<&mut zenoh_flow::DataMessage> for DataMessage {
    type Error = ZFError;
    fn try_from(msg: &mut zenoh_flow::DataMessage) -> Result<Self, Self::Error> {
        let missed_end_to_end_deadlines: Vec<E2EDeadlineMiss> = msg
            .get_missed_end_to_end_deadlines()
            .iter()
            .map(|e2e_deadline| e2e_deadline.into())
            .collect();

        Ok(Self {
            ts: *msg.get_timestamp(),
            data: msg.get_inner_data().try_as_bytes()?.to_vec(),
            missed_end_to_end_deadlines,
        })
    }
}

/// The inputs received in the Operator run function.
#[pyclass]
#[derive(Clone)]
pub struct Inputs {
    pub(crate) inputs: HashMap<String, DataMessage>,
}

#[pymethods]
impl Inputs {
    /// Gets the :class:`DataMessage` from the :class:`Inputs`
    ///
    /// :param id: The ID of the input port.
    /// :type id: str
    ///
    /// :rtype: :class:`DataMessage`
    fn get(&self, id: String) -> Option<DataMessage> {
        self.inputs.get(&id).cloned()
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Total data {}", self.inputs.len()))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}

impl TryFrom<&mut HashMap<zenoh_flow::PortId, zenoh_flow::DataMessage>> for Inputs {
    type Error = ZFError;
    fn try_from(
        rust_inputs: &mut HashMap<zenoh_flow::PortId, zenoh_flow::DataMessage>,
    ) -> Result<Self, Self::Error> {
        let mut inputs = HashMap::new();
        for (k, v) in rust_inputs {
            let port_id = k.to_string();
            let data = DataMessage::try_from(v)?;
            inputs.insert(port_id, data);
        }
        Ok(Self { inputs })
    }
}

/// Zenoh Flow outputs, passed to the operator output rules
#[pyclass]
#[derive(Clone)]
pub struct Outputs {
    pub(crate) outputs: HashMap<String, Vec<u8>>,
}

#[pymethods]
impl Outputs {
    #[new]
    fn new() -> Self {
        Self {
            outputs: HashMap::new(),
        }
    }
    /// Adds a value to the :class:`Outputs`
    ///
    /// :param id: the ID of the output port
    /// :type id: str
    /// :param data: The data
    /// :type id: bytes
    ///
    fn put(&mut self, id: String, data: Vec<u8>) {
        self.outputs.insert(id, data);
    }

    /// Gets the data from the :class:`Outputs
    /// `
    /// :param id: The ID of the output port.
    /// :type id: str
    ///
    /// :rtype: bytes
    fn get(&self, id: String) -> Option<Vec<u8>> {
        self.outputs.get(&id).cloned()
    }

    fn __str__(&self) -> PyResult<String> {
        Ok(format!("Total data {}", self.outputs.len()))
    }

    fn __repr__(&self) -> PyResult<String> {
        self.__str__()
    }
}

impl IntoIterator for Outputs {
    type Item = (String, Vec<u8>);
    type IntoIter = std::collections::hash_map::IntoIter<String, Vec<u8>>;

    fn into_iter(self) -> Self::IntoIter {
        self.outputs.into_iter()
    }
}

impl TryFrom<HashMap<zenoh_flow::PortId, zenoh_flow::Data>> for Outputs {
    type Error = ZFError;
    fn try_from(
        rust_inputs: HashMap<zenoh_flow::PortId, zenoh_flow::Data>,
    ) -> Result<Self, Self::Error> {
        let mut outputs = HashMap::new();
        for (k, v) in rust_inputs {
            let port_id = k.to_string();
            let data = v.try_as_bytes()?.to_vec();
            outputs.insert(port_id, data);
        }
        Ok(Self { outputs })
    }
}

impl TryFrom<(Py<PyDict>, Python<'_>)> for Outputs {
    type Error = ZFError;
    fn try_from(dict: (Py<PyDict>, Python<'_>)) -> Result<Self, Self::Error> {
        let (dict, py) = dict;
        let mut outputs = HashMap::new();
        for (k, v) in dict.as_ref(py).into_iter() {
            let port_id = k.to_string();
            let data: Vec<u8> = v
                .extract()
                .map_err(|e| ZFError::InvalidData(e.to_string()))?;
            outputs.insert(port_id, data);
        }
        Ok(Self { outputs })
    }
}

impl TryInto<HashMap<zenoh_flow::PortId, zenoh_flow::Data>> for Outputs {
    type Error = ZFError;
    fn try_into(self) -> Result<HashMap<zenoh_flow::PortId, zenoh_flow::Data>, Self::Error> {
        let mut outputs = HashMap::new();
        for (k, v) in self.outputs {
            let data = zenoh_flow::Data::from_bytes(v);
            outputs.insert(k.into(), data);
        }
        Ok(outputs)
    }
}

/// A Zenoh Flow Input Token
#[pyclass]
#[derive(Clone, Debug)]
pub struct InputToken {
    pub(crate) token: zenoh_flow::InputToken,
}

#[pymethods]
impl InputToken {
    /// Sets the token to be dropped.
    pub fn set_action_drop(&mut self) {
        self.token.set_action_drop()
    }

    /// Sets the token to be kept for next iteration.
    pub fn set_action_keep(&mut self) {
        self.token.set_action_keep()
    }

    /// Sets the token to be consumed in the current iteration (default).
    pub fn set_action_consume(&mut self) {
        self.token.set_action_consume()
    }

    /// Gets the timestamp from the :class:`Token`.
    ///
    /// :rtype: str
    pub fn get_timestamp(&self) -> String {
        match &self.token {
            zenoh_flow::InputToken::Ready(ref r) => r.get_timestamp().to_string(),
            _ => String::from(""),
        }
    }

    /// Gets the data from the :class:`Token`
    ///
    /// :rtype: bytes
    pub fn get_data(&mut self) -> PyResult<Vec<u8>> {
        match &mut self.token {
            zenoh_flow::InputToken::Ready(ref mut r) => {
                let data = r.get_data_mut();
                Ok(data
                    .try_as_bytes()
                    .map_err(|_| {
                        pyo3::exceptions::PyValueError::new_err("Unable to get data from token")
                    })?
                    .to_vec())
            }
            _ => Err(pyo3::exceptions::PyValueError::new_err(
                "Pending Token has no data",
            )),
        }
    }

    /// Gets the action from the :class:`Token`
    ///
    /// :rtype: str
    pub fn get_action(&self) -> String {
        match &self.token {
            zenoh_flow::InputToken::Ready(ref r) => r.get_action().to_string(),
            _ => String::from("Pending"),
        }
    }

    /// Checks if the :class:`Token` is ready.
    /// i.e. has Data.
    ///
    /// :rtype: bool
    pub fn is_ready(&self) -> bool {
        matches!(&self.token, zenoh_flow::InputToken::Ready(_))
    }

    /// Checks if the :class:`Token` is pending.
    /// i.e. has no data.
    ///
    /// :rtype: bool
    pub fn is_pending(&self) -> bool {
        matches!(&self.token, zenoh_flow::InputToken::Pending)
    }
}

impl From<zenoh_flow::InputToken> for InputToken {
    fn from(token: zenoh_flow::InputToken) -> Self {
        Self { token }
    }
}

impl From<InputToken> for zenoh_flow::InputToken {
    fn from(val: InputToken) -> Self {
        val.token
    }
}

/// A set of :class:`InputTokens`
#[pyclass]
#[derive(Clone, Debug)]
pub struct InputTokens {
    pub(crate) tokens: HashMap<String, InputToken>,
}

#[pymethods]
impl InputTokens {
    /// Gets the :class:`InputToken` for the given port ID.
    ///
    /// :param port_id: The input port ID.
    /// :type port_id: str
    ///
    /// :rtype: :class:`InputToken`
    pub fn get(&mut self, port_id: String) -> PyResult<InputToken> {
        match self.tokens.get(&port_id) {
            Some(t) => Ok(t.clone()),
            None => Err(pyo3::exceptions::PyValueError::new_err(format!(
                "No tokens found for the given port_id {}",
                port_id
            ))),
        }
    }
}

// impl From<zenoh_flow::InputTokens> for Tokens {
//     fn from(tokens: zenoh_flow::InputTokens) -> Self {
//         Self { tokens }
//     }
// }

impl From<HashMap<zenoh_flow::PortId, zenoh_flow::InputToken>> for InputTokens {
    fn from(rust_tokens: HashMap<zenoh_flow::PortId, zenoh_flow::InputToken>) -> Self {
        Self {
            tokens: rust_tokens
                .into_iter()
                .map(|(id, token)| (id.to_string(), InputToken::from(token)))
                .collect(),
        }
    }
}

impl From<InputTokens> for HashMap<zenoh_flow::PortId, zenoh_flow::InputToken> {
    fn from(val: InputTokens) -> Self {
        val.tokens
            .into_iter()
            .map(|(id, token)| (id.into(), token.into()))
            .collect()
    }
}

/// A Zenoh Flow local deadline miss.
/// A structure containing all the information regarding a missed, local, deadline.
/// A local deadline is represented by a maximum time between receiving the
/// data at the Input Rules and providing a result to the Output Rule.
/// This means that if the Run function takes more that the deadline
/// the Output Rule will be notified by the means of this
/// `LocalDeadlineMiss`.
#[pyclass]
#[derive(Clone)]
pub struct LocalDeadlineMiss {
    pub(crate) deadline: u128,
    pub(crate) elapsed: u128,
}

#[pymethods]
impl LocalDeadlineMiss {
    /// Gets the deadline.
    ///
    /// :rtype: int
    #[getter]
    fn deadline(&self) -> u128 {
        self.deadline
    }

    /// Gets the elapsed time.
    ///
    /// :rtype: int
    #[getter]
    fn elapsed(&self) -> u128 {
        self.elapsed
    }
}

impl From<zenoh_flow::LocalDeadlineMiss> for LocalDeadlineMiss {
    fn from(deadline_miss: zenoh_flow::LocalDeadlineMiss) -> Self {
        Self {
            deadline: deadline_miss.deadline.as_micros(),
            elapsed: deadline_miss.elapsed.as_micros(),
        }
    }
}

impl From<Option<zenoh_flow::LocalDeadlineMiss>> for LocalDeadlineMiss {
    fn from(deadline_miss: Option<zenoh_flow::LocalDeadlineMiss>) -> Self {
        match deadline_miss {
            Some(dl_miss) => Self {
                deadline: dl_miss.deadline.as_micros(),
                elapsed: dl_miss.elapsed.as_micros(),
            },
            None => Self {
                deadline: 0,
                elapsed: 0,
            },
        }
    }
}

/// The descriptor on where an E2E Deadline starts.
///
#[pyclass]
#[derive(Clone)]
pub struct FromDescriptor {
    pub node: String,
    pub output: String,
}

#[pymethods]
impl FromDescriptor {
    /// Gets the node ID from :class:`FromDescriptor`
    ///
    /// :rtype: str
    #[getter]
    fn node(&self) -> &str {
        &self.node
    }

    /// Gets the port ID from :class:`FromDescriptor`
    ///
    /// :rtype: str
    #[getter]
    fn output(&self) -> &str {
        &self.output
    }
}

/// The descriptor on where a E2E Deadline ends.
///
#[pyclass]
#[derive(Clone)]
pub struct ToDescriptor {
    pub node: String,
    pub input: String,
}

#[pymethods]
impl ToDescriptor {
    /// Gets the node ID from :class:`ToDescriptor`
    ///
    /// :rtype: str
    #[getter]
    fn node(&self) -> &str {
        &self.node
    }

    /// Gets the port ID from :class:`ToDescriptor`
    ///
    /// :rtype: str
    #[getter]
    fn input(&self) -> &str {
        &self.input
    }
}

/// A End to End Deadline.
/// A deadline can apply for a whole graph or for a subpart of it.
#[pyclass]
#[derive(Clone)]
pub struct E2EDeadlineMiss {
    pub from: FromDescriptor,
    pub to: ToDescriptor,
    pub start: u64,
    pub end: u64,
}

#[pymethods]
impl E2EDeadlineMiss {
    /// Gets from where the deadline starts.
    ///
    /// :rtype: :class:`FromDescriptor`
    #[getter]
    fn from(&self) -> FromDescriptor {
        self.from.clone()
    }

    /// Gets where the deadline ends.
    ///
    /// :rtype: :class:`ToDescriptor`
    #[getter]
    fn to(&self) -> ToDescriptor {
        self.to.clone()
    }

    /// Gets the start time of the deadline.
    /// :rtype: int
    #[getter]
    fn start(&self) -> u64 {
        self.start
    }
    /// Gets the end time of the deadline.
    ///
    /// :rtype: int
    #[getter]
    fn end(&self) -> u64 {
        self.end
    }
}

impl From<&zenoh_flow::runtime::deadline::E2EDeadlineMiss> for E2EDeadlineMiss {
    fn from(e2d_deadline_miss: &zenoh_flow::runtime::deadline::E2EDeadlineMiss) -> Self {
        let to = ToDescriptor {
            node: e2d_deadline_miss.to.node.to_string(),
            input: e2d_deadline_miss.to.input.to_string(),
        };
        let from = FromDescriptor {
            node: e2d_deadline_miss.from.node.to_string(),
            output: e2d_deadline_miss.from.output.to_string(),
        };

        Self {
            from,
            to,
            start: e2d_deadline_miss.start.get_time().as_u64(),
            end: e2d_deadline_miss.end.get_time().as_u64(),
        }
    }
}
