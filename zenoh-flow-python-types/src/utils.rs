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

use crate::{InputToken, Outputs};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList};
use std::collections::HashMap;
use std::convert::TryFrom;
use zenoh_flow::{ZFError, ZFResult};

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
        //FIXME: it should convert to the right type: i64, u64 or f64
        zenoh_flow::Configuration::Number(n) => Ok(n.as_u64().unwrap().to_object(py)),
        zenoh_flow::Configuration::String(s) => Ok(s.to_object(py)),
        zenoh_flow::Configuration::Null => Ok(py.None()),
    }
}

pub fn tokens_into_py(
    py: Python,
    rust_tokens: HashMap<zenoh_flow::PortId, zenoh_flow::InputToken>,
) -> PyObject {
    let mut tokens: HashMap<String, InputToken> = HashMap::new();

    for (id, t) in rust_tokens {
        tokens.insert(id.as_ref().clone().into(), InputToken::from(t));
    }

    tokens.into_py(py)
}

pub fn outputs_from_py(py: Python, obj: PyObject) -> ZFResult<Outputs> {
    match obj
        .extract::<&PyDict>(py)
        .map_err(|e| ZFError::InvalidData(e.to_string()))
    {
        Ok(dict) => {
            let dict: Py<PyDict> = dict.into();
            let values = Outputs::try_from((dict, py))?;
            Ok(values)
        }
        Err(_) => {
            let values: ZFResult<Outputs> = obj
                .extract(py)
                .map_err(|e| ZFError::InvalidData(e.to_string()));
            values
        }
    }
}
