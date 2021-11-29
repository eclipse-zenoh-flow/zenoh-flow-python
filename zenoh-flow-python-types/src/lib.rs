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

use pyo3::prelude::*;
use pyo3::types::PyDict;
use pyo3::PyObjectProtocol;
use std::collections::HashMap;
use std::convert::TryInto;
use std::convert::{From, TryFrom};
use zenoh_flow::ZFError;

pub mod utils;

#[pyclass]
pub struct Context {
    pub(crate) mode: usize,
}

#[pymethods]
impl Context {
    #[getter]
    fn mode(&self) -> usize {
        self.mode.clone()
    }
}

impl From<&mut zenoh_flow::Context> for Context {
    fn from(ctx: &mut zenoh_flow::Context) -> Self {
        Self {
            mode: ctx.mode.clone(),
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct DataMessage {
    pub(crate) data: Vec<u8>,
    pub(crate) ts: uhlc::Timestamp,
    pub(crate) missed_end_to_end_deadlines: Vec<E2EDeadlineMiss>,
}

#[pymethods]
impl DataMessage {
    #[getter]
    fn timestamp(&self) -> String {
        self.ts.to_string()
    }

    #[getter]
    fn data(&self) -> &[u8] {
        &self.data
    }
}

#[pyproto]
impl PyObjectProtocol for DataMessage {
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
            ts: msg.get_timestamp().clone(),
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
            ts: msg.get_timestamp().clone(),
            data: msg.get_inner_data().try_as_bytes()?.to_vec(),
            missed_end_to_end_deadlines,
        })
    }
}

#[pyclass]
#[derive(Clone)]
pub struct Inputs {
    pub(crate) inputs: HashMap<String, DataMessage>,
}

#[pymethods]
impl Inputs {
    fn get(&self, id: String) -> Option<DataMessage> {
        match self.inputs.get(&id) {
            Some(dm) => Some(dm.clone()),
            None => None,
        }
    }
}

#[pyproto]
impl PyObjectProtocol for Inputs {
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

    fn put(&mut self, id: String, data: Vec<u8>) -> () {
        self.outputs.insert(id, data);
    }
}

#[pyproto]
impl PyObjectProtocol for Outputs {
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
    fn try_into(self: Self) -> Result<HashMap<zenoh_flow::PortId, zenoh_flow::Data>, Self::Error> {
        let mut outputs = HashMap::new();
        for (k, v) in self.outputs {
            let data = zenoh_flow::Data::from_bytes(v);
            outputs.insert(k.into(), data);
        }
        Ok(outputs)
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct Token {
    pub(crate) token: zenoh_flow::Token,
}

#[pymethods]
impl Token {
    pub fn set_action_drop(&mut self) {
        match &mut self.token {
            zenoh_flow::Token::Ready(ref mut r) => r.set_action_drop(),
            _ => (),
        }
    }

    pub fn set_action_keep(&mut self) {
        match &mut self.token {
            zenoh_flow::Token::Ready(ref mut r) => r.set_action_keep(),
            _ => (),
        }
    }

    pub fn set_action_consume(&mut self) {
        match &mut self.token {
            zenoh_flow::Token::Ready(ref mut r) => r.set_action_consume(),
            _ => (),
        }
    }

    pub fn get_timestamp(&self) -> String {
        match &self.token {
            zenoh_flow::Token::Ready(ref r) => r.get_timestamp().to_string(),
            _ => String::from(""),
        }
    }

    pub fn get_data(&mut self) -> PyResult<Vec<u8>> {
        match &mut self.token {
            zenoh_flow::Token::Ready(ref mut r) => {
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

    pub fn get_action(&self) -> String {
        match &self.token {
            zenoh_flow::Token::Ready(ref r) => r.get_action().to_string(),
            _ => String::from("Pending"),
        }
    }
}

impl From<zenoh_flow::Token> for Token {
    fn from(token: zenoh_flow::Token) -> Self {
        Self { token }
    }
}

impl Into<zenoh_flow::Token> for Token {
    fn into(self) -> zenoh_flow::Token {
        self.token
    }
}

#[pyclass]
#[derive(Clone, Debug)]
pub struct Tokens {
    pub(crate) tokens: HashMap<String, Token>,
}

#[pymethods]
impl Tokens {
    pub fn get(&mut self, port_id: String) -> PyResult<Token> {
        match self.tokens.get(&port_id) {
            Some(t) => Ok(t.clone()),
            None => Err(pyo3::exceptions::PyValueError::new_err(
                format!("No tokens found for the given port_id {}", port_id).to_string(),
            )),
        }
    }
}

// impl From<zenoh_flow::Tokens> for Tokens {
//     fn from(tokens: zenoh_flow::Tokens) -> Self {
//         Self { tokens }
//     }
// }

impl From<HashMap<zenoh_flow::PortId, zenoh_flow::Token>> for Tokens {
    fn from(rust_tokens: HashMap<zenoh_flow::PortId, zenoh_flow::Token>) -> Self {
        let mut tokens = HashMap::new();

        for (id, t) in rust_tokens {
            tokens.insert(id.as_ref().clone().into(), Token::from(t));
        }

        Self { tokens }
    }
}

impl Into<HashMap<zenoh_flow::PortId, zenoh_flow::Token>> for Tokens {
    fn into(self) -> HashMap<zenoh_flow::PortId, zenoh_flow::Token> {
        let mut tokens = HashMap::new();

        for (id, t) in self.tokens {
            tokens.insert(id.into(), t.into());
        }

        tokens
    }
}

#[pyclass]
#[derive(Clone)]
pub struct LocalDeadlineMiss {
    pub(crate) deadline: u128,
    pub(crate) elapsed: u128,
}

#[pymethods]
impl LocalDeadlineMiss {
    #[getter]
    fn deadline(&self) -> u128 {
        self.deadline
    }

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

#[pyclass]
#[derive(Clone)]
pub struct FromDescriptor {
    pub node: String,
    pub output: String,
}

#[pymethods]
impl FromDescriptor {
    #[getter]
    fn node(&self) -> &str {
        &self.node
    }

    #[getter]
    fn output(&self) -> &str {
        &self.output
    }
}

#[pyclass]
#[derive(Clone)]
pub struct ToDescriptor {
    pub node: String,
    pub input: String,
}

#[pymethods]
impl ToDescriptor {
    #[getter]
    fn node(&self) -> &str {
        &self.node
    }

    #[getter]
    fn input(&self) -> &str {
        &self.input
    }
}

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
    #[getter]
    fn from(&self) -> FromDescriptor {
        self.from.clone()
    }

    #[getter]
    fn to(&self) -> ToDescriptor {
        self.to.clone()
    }

    #[getter]
    fn start(&self) -> u64 {
        self.start
    }

    #[getter]
    fn end(&self) -> u64 {
        self.end
    }
}

impl From<&zenoh_flow::runtime::deadline::E2EDeadlineMiss> for E2EDeadlineMiss {
    fn from(e2d_deadline_miss: &zenoh_flow::runtime::deadline::E2EDeadlineMiss) -> Self {
        let to = ToDescriptor {
            node: e2d_deadline_miss.to.node.as_ref().clone().into(),
            input: e2d_deadline_miss.to.input.as_ref().clone().into(),
        };
        let from = FromDescriptor {
            node: e2d_deadline_miss.from.node.as_ref().clone().into(),
            output: e2d_deadline_miss.from.output.as_ref().clone().into(),
        };

        Self {
            from,
            to,
            start: e2d_deadline_miss.start.get_time().as_u64(),
            end: e2d_deadline_miss.end.get_time().as_u64(),
        }
    }
}
