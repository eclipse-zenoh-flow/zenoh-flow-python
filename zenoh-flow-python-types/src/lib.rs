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
use pyo3::types::{PyDict, PyList};
use pyo3::PyObjectProtocol;
use std::collections::HashMap;
use std::convert::TryInto;
use std::convert::{From, TryFrom};
use zenoh_flow::ZFError;
pub fn into_py(py: Python, value: zenoh_flow::Configuration) -> PyObject {
    match value {
        zenoh_flow::Configuration::Array(arr) => {
            let py_list = PyList::empty(py);
            for v in arr {
                py_list.append(into_py(py, v)).unwrap();
            }
            py_list.to_object(py)
        }
        zenoh_flow::Configuration::Object(obj) => {
            let py_dict = PyDict::new(py);
            for (k, v) in obj {
                py_dict.set_item(k, into_py(py, v)).unwrap();
            }
            py_dict.to_object(py)
        }
        zenoh_flow::Configuration::Bool(b) => b.to_object(py),
        zenoh_flow::Configuration::Number(n) => n.as_u64().unwrap().to_object(py), //TODO convert to the right number
        zenoh_flow::Configuration::String(s) => s.to_object(py),
        zenoh_flow::Configuration::Null => py.None(),
    }
}

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
    fn try_from(msg: zenoh_flow::DataMessage) -> Result<Self, Self::Error> {
        Ok(Self {
            ts: msg.timestamp,
            data: msg.data.try_as_bytes()?.to_vec(),
        })
    }
}

impl TryFrom<&mut zenoh_flow::DataMessage> for DataMessage {
    type Error = ZFError;
    fn try_from(msg: &mut zenoh_flow::DataMessage) -> Result<Self, Self::Error> {
        Ok(Self {
            ts: msg.timestamp,
            data: msg.data.try_as_bytes()?.to_vec(),
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
            let data : Vec<u8> = v.extract().map_err(|e| ZFError::InvalidData(e.to_string()))?;
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
