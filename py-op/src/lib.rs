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

use pyo3::{prelude::*, types::{PyModule, PyDict}};
use std::collections::HashMap;
use std::convert::TryFrom;
use std::convert::TryInto;
use std::fs;
use std::path::Path;
use zenoh_flow::async_std::sync::Arc;
use zenoh_flow::runtime::message::DataMessage;
use zenoh_flow::zenoh_flow_derive::ZFState;
use zenoh_flow::Configuration;
use zenoh_flow::{
    default_input_rule, default_output_rule, export_operator, types::ZFResult, Node, NodeOutput,
    Operator, PortId, State, Token,
};
use zenoh_flow::{Context, Data, ZFError};
use zenoh_flow_python_types::into_py;
use zenoh_flow_python_types::Context as PyContext;
use zenoh_flow_python_types::Inputs as PyInputs;
use zenoh_flow_python_types::Outputs as PyOutputs;

#[derive(ZFState, Clone)]
struct PythonState {
    pub module: Arc<PyObject>,
    pub py_state: Arc<PyObject>,
}
unsafe impl Send for PythonState {}
unsafe impl Sync for PythonState {}

impl std::fmt::Debug for PythonState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PythonState").finish()
    }
}

struct PyOperator;

impl Operator for PyOperator {
    fn input_rule(
        &self,
        _context: &mut Context,
        state: &mut State,
        tokens: &mut HashMap<PortId, Token>,
    ) -> ZFResult<bool> {
        default_input_rule(state, tokens)
    }

    fn run(
        &self,
        ctx: &mut Context,
        state: &mut State,
        inputs: &mut HashMap<PortId, DataMessage>,
    ) -> ZFResult<HashMap<PortId, Data>> {
        let gil = Python::acquire_gil();
        let py = gil.python();
        let current_state = state.try_get::<PythonState>()?;
        let op_class = current_state.module.as_ref().clone();

        let py_ctx = PyContext::from(ctx);
        let py_data = PyInputs::try_from(inputs)?;
        let py_values = op_class
            .call_method1(
                py,
                "run",
                (
                    op_class.clone(),
                    py_ctx,
                    current_state.py_state.as_ref().clone(),
                    py_data,
                ),
            )
            .map_err(|e| ZFError::InvalidData(e.to_string()))?;

        // println!("Python values {:?}", py_values);
        // println!(
        //     "Is outputs? {:?}",
        //     py_values.as_ref(py).is_instance::<PyOutputs>()
        // );
        // println!("Python Values type? {:?}", py_values.as_ref(py).get_type());

        let values : PyObject = py_values.into();
        let dict : &PyDict = values.extract(py).map_err(|e| ZFError::InvalidData(e.to_string()))?;
        let dict : Py<PyDict> = dict.into();
        let values = PyOutputs::try_from((dict, py))?;


        // let dict : &PyDict = py_values.extract(py).map_err(|e| ZFError::InvalidData(e.to_string()))?;
        // let dict : Py<PyDict> = dict.into();
        // let values = PyOutputs::try_from((dict, py))?;

        Ok(values.try_into()?)
    }

    fn output_rule(
        &self,
        _context: &mut Context,
        state: &mut State,
        outputs: HashMap<zenoh_flow::PortId, Data>,
    ) -> ZFResult<HashMap<zenoh_flow::PortId, NodeOutput>> {
        default_output_rule(state, outputs)
    }
}

impl Node for PyOperator {
    fn initialize(&self, configuration: &Option<Configuration>) -> ZFResult<State> {
        pyo3::prepare_freethreaded_python();
        let gil = Python::acquire_gil();
        let py = gil.python();
        match configuration {
            Some(configuration) => {
                let script_file_path = Path::new(
                    configuration["python-script"]
                        .as_str()
                        .ok_or(ZFError::InvalidState)?,
                );
                let mut config = configuration.clone();
                config["python-script"].take();

                let py_config = into_py(py, config);

                let code = read_file(script_file_path);
                let module = PyModule::from_code(py, &code, "op.py", "op")
                    .map_err(|e| ZFError::InvalidData(e.to_string()))?;

                let op_class: PyObject = module
                    .call_method0("register")
                    .map_err(|e| ZFError::InvalidData(e.to_string()))?
                    .into();

                let state: PyObject = op_class
                    .call_method1(py, "initialize", (op_class.clone(), py_config))
                    .map_err(|e| ZFError::InvalidData(e.to_string()))?
                    .into();

                Ok(State::from(PythonState {
                    module: Arc::new(op_class),
                    py_state: Arc::new(state),
                }))
            }
            None => Err(ZFError::InvalidState),
        }
    }

    fn finalize(&self, _state: &mut State) -> ZFResult<()> {
        Ok(())
    }
}

export_operator!(register);

fn register() -> ZFResult<Arc<dyn Operator>> {
    Ok(Arc::new(PyOperator) as Arc<dyn Operator>)
}

fn read_file(path: &Path) -> String {
    fs::read_to_string(path).unwrap()
}
