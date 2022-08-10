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
#![feature(async_closure)]

use async_trait::async_trait;
use pyo3::types::{PyDict, PyString};
use pyo3::{prelude::*, types::PyModule};
use pyo3_asyncio::TaskLocals;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use zenoh_flow::{
    export_operator, types::ZFResult, AsyncIteration, Configuration, Context, Inputs, Node,
    Operator, Outputs, ZFError,
};
use zenoh_flow_python_common::from_pyerr_to_zferr;
use zenoh_flow_python_common::PythonState;
use zenoh_flow_python_common::{configuration_into_py, DataReceiver, DataSender};

#[cfg(target_family = "unix")]
use libloading::os::unix::Library;
#[cfg(target_family = "windows")]
use libloading::Library;

#[cfg(target_family = "unix")]
static LOAD_FLAGS: std::os::raw::c_int =
    libloading::os::unix::RTLD_NOW | libloading::os::unix::RTLD_GLOBAL;

pub static PY_LIB: &str = env!("PY_LIB");

struct PyOperator(Library);

#[async_trait]
impl Operator for PyOperator {
    async fn setup(
        &self,
        _ctx: &mut Context,
        configuration: &Option<Configuration>,
        inputs: Inputs,
        outputs: Outputs,
    ) -> ZFResult<Option<Arc<dyn AsyncIteration>>>  {
        let my_state = Python::with_gil(|py| {
            match configuration {
                Some(configuration) => {
                    // Unwrapping configuration
                    let script_file_path = Path::new(
                        configuration["python-script"]
                            .as_str()
                            .ok_or(ZFError::InvalidState)?,
                    );
                    let mut config = configuration.clone();
                    config["python-script"].take();
                    let py_config = config["configuration"].take();

                    // Convert configuration to Python
                    let py_config = configuration_into_py(py, py_config)
                        .map_err(|e| from_pyerr_to_zferr(e, &py))?;

                    // Load Python code
                    let code = read_file(script_file_path)?;
                    let module =
                        PyModule::from_code(py, &code, &script_file_path.to_string_lossy(), "op")
                            .map_err(|e| from_pyerr_to_zferr(e, &py))?;

                    // Getting the correct python module
                    let op_class = module
                        .call_method0("register")
                        .map_err(|e| from_pyerr_to_zferr(e, &py))?;

                    let py_receivers = PyDict::new(py);

                    for (id, input) in inputs.into_iter() {
                        let pyo3_rx = DataReceiver::from(input);
                        py_receivers
                            .set_item(PyString::new(py, &id), &pyo3_rx.into_py(py))
                            .map_err(|e| from_pyerr_to_zferr(e, &py))?;
                    }

                    let py_senders = PyDict::new(py);

                    for (id, output) in outputs.into_iter() {
                        let pyo3_tx = DataSender::from(output);
                        py_senders
                            .set_item(PyString::new(py, &id), &pyo3_tx.into_py(py))
                            .map_err(|e| from_pyerr_to_zferr(e, &py))?;
                    }

                    // Setting asyncio event loop
                    let asyncio = py.import("asyncio").unwrap();

                    let event_loop = asyncio.call_method0("new_event_loop").unwrap();
                    asyncio
                        .call_method1("set_event_loop", (event_loop,))
                        .unwrap();
                    let event_loop_hdl = Arc::new(PyObject::from(event_loop));

                    // Initialize python state
                    let lambda: PyObject = op_class
                        .call_method1("setup", (op_class, py_config, py_receivers, py_senders))
                        .map_err(|e| from_pyerr_to_zferr(e, &py))?
                        .into();

                    Ok(PythonState {
                        module: Arc::new(op_class.into()),
                        py_state: Arc::new(lambda),
                        event_loop: event_loop_hdl,
                        asyncio_module: Arc::new(PyObject::from(asyncio)),
                    })
                }
                None => Err(ZFError::InvalidState),
            }
        })?;

        Ok(Some(Arc::new(async move || {
            Python::with_gil(|py| {
                let py_state = my_state.py_state.cast_as::<PyAny>(py)?;

                let event_loop = my_state.event_loop.cast_as::<PyAny>(py)?;

                let task_locals = TaskLocals::new(event_loop);

                let py_future = <&pyo3::PyAny>::clone(&py_state.call0()?);

                let fut = pyo3_asyncio::into_future_with_locals(&task_locals, py_future)?;
                pyo3_asyncio::async_std::run_until_complete(event_loop, fut)
            })
            .map_err(|e| Python::with_gil(|py| from_pyerr_to_zferr(e, &py)))?;
            Ok(())
        })))
    }
}
#[async_trait]
impl Node for PyOperator {
    async fn finalize(&self) -> ZFResult<()> {
        // let gil = Python::acquire_gil();
        // let py = gil.python();
        // let current_state = state.try_get::<PythonState>()?;

        // let py_op = current_state
        //     .module
        //     .cast_as::<PyAny>(py)
        //     .map_err(|e| from_pyerr_to_zferr(e.into(), &py))?;

        // let py_state = current_state
        //     .py_state
        //     .as_ref()
        //     .cast_as::<PyAny>(py)
        //     .map_err(|e| from_pyerr_to_zferr(e.into(), &py))?;

        // py_op
        //     .call_method1("finalize", (py_op, py_state))
        //     .map_err(|e| from_pyerr_to_zferr(e, &py))?;

        Ok(())
    }
}

export_operator!(register);

fn load_self() -> ZFResult<Library> {
    log::trace!("Python Operator Wrapper loading Python {}", PY_LIB);
    // Very dirty hack! We explicit load the python library!
    let lib_name = libloading::library_filename(PY_LIB);
    unsafe {
        #[cfg(target_family = "unix")]
        let lib = Library::open(Some(lib_name), LOAD_FLAGS)?;

        #[cfg(target_family = "windows")]
        let lib = Library::new(lib_name)?;

        Ok(lib)
    }
}

fn register() -> ZFResult<Arc<dyn Operator>> {
    let library = load_self()?;

    Ok(Arc::new(PyOperator(library)) as Arc<dyn Operator>)
}

fn read_file(path: &Path) -> ZFResult<String> {
    Ok(fs::read_to_string(path)?)
}
