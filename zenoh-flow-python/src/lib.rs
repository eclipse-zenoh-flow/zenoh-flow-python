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

use pyo3::types::PyDict;
use pyo3::{prelude::*, types::PyModule};
use std::collections::HashMap;
pub use zenoh_flow_python_types::{
    Context, DataMessage, InputToken, Inputs, LocalDeadlineMiss, Outputs,
};

/// The class representing a Zenoh Flow source
#[pyclass(subclass)]
pub struct Source {}

#[pymethods]
impl Source {
    #[new]
    fn new() -> Self {
        Source {}
    }
    /// The run method is called by the zenoh flow runtime.
    /// This method is expected to produce data whenever it is called.
    /// Any source has to implement this method.
    ///
    /// :rtype: bytes
    fn run(&self, _state: Py<PyAny>) -> PyResult<Vec<u8>> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }

    /// The initialize method is called by the zenoh flow runtime.
    /// This method is called when starting the data flow graph.
    /// Any source has to implement this method.
    /// This method is use to initialize any state that can be useful
    /// for the source (e.g. open files)
    /// It should then return the state to the runtime.
    ///
    /// :param configuration: Configuration
    /// :type configuraion: dict
    ///
    /// :rtype: any
    fn initialize(&self, _configuration: Option<PyObject>) -> PyResult<Py<PyAny>> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }

    /// The finalize method is called by the zenoh flow runtime.
    /// This method is called when stopping the data flow graph.
    /// Any source has to implement this method.
    /// This method is use to finalize any state that can be useful
    /// for the source (e.g. close files)
    /// It should destroy the state.
    ///
    /// :param state: Source state
    /// :type state: any
    fn finalize(&self, _state: Py<PyAny>) -> PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }
}

/// The class representing a Zenoh Flow sink
#[pyclass(subclass)]
pub struct Sink {}

#[pymethods]
impl Sink {
    #[new]
    fn new() -> Self {
        Self {}
    }

    /// The run method is called by the Zenoh Flow runtime.
    /// Any sink has to implement this method.
    /// This method is called when data is received from the input.
    ///
    /// :param context: The Sink context
    /// :type context: :class:`Context`
    /// :param state: The sink state
    /// :type state: any
    /// :param input: The data message.
    /// :type input: :class:`DataMessage`
    ///
    ///
    fn run(&self, _context: &mut Context, _state: Py<PyAny>, _input: DataMessage) -> PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }

    /// The initialize method is called by the zenoh flow runtime.
    /// This method is called when starting the data flow graph.
    /// Any sink has to implement this method.
    /// This method is use to initialize any state that can be useful
    /// for the sink (e.g. open files)
    /// It should then return the state to the runtime.
    ///
    /// :param configuration: Configuration
    /// :type configuraion: dict
    ///
    /// :rtype: any
    fn initialize(&self, _configuration: Option<PyObject>) -> PyResult<Py<PyAny>> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }

    /// The finalize method is called by the zenoh flow runtime.
    /// This method is called when stopping the data flow graph.
    /// Any sink has to implement this method.
    /// This method is use to finalize any state that can be useful
    /// for the sink (e.g. close files)
    /// It should destroy the state.
    ///
    /// :param state: Sink state
    /// :type state: any
    ///
    fn finalize(&self, _state: Py<PyAny>) -> PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }
}

#[pyclass(subclass)]
pub struct Operator {}

#[pymethods]
impl Operator {
    #[new]
    fn new() -> Self {
        Self {}
    }

    /// The input_rule method is called by the zenoh flow runtime.
    /// This method is called when data is received on one or more inputs.
    /// This result of this method is used as discriminant to trigger the
    /// run function.
    /// Any operator has to implement this method.
    ///
    /// :param context: The operator context
    /// :type context: :class:`Context`
    /// :param state: The source state
    /// :type state: any
    /// :param tokens: Tokens received from inputs
    /// :type tokens: dict{str,:class:`InputToken`}
    ///
    /// :rtype: bool
    fn input_rule(
        &self,
        _context: &mut Context,
        _state: Py<PyAny>,
        _tokens: HashMap<String, InputToken>,
    ) -> PyResult<bool> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }

    /// The run method is called by the zenoh flow runtime.
    /// This method is called when the result of input_rule is true.
    /// This result of this method is used as the `outputs` parameter for the
    /// output_rule function.
    /// Any operator has to implement this method.
    ///
    /// :param context: The operator context
    /// :type context: :class:`Context`
    /// :param state: The source state
    /// :type state: any
    /// :param inputs: Input data received from the inputs
    /// :type inputs: :class:`Inputs`
    ///
    /// :rtype: dict{str, bytes}
    fn run(
        &self,
        _context: &mut Context,
        _state: Py<PyAny>,
        _inputs: Inputs,
    ) -> PyResult<Py<PyDict>> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }

    /// The output_rule method is called by the zenoh flow runtime.
    /// This method is called when data is produced from the run.
    /// Any operator has to implement this method.
    ///
    /// :param context: The operator context
    /// :type context: :class:`Context`
    /// :param state: The source state
    /// :type state: any
    /// :param outputs: The outputs generated by the run.
    /// :type outputs: :class:`Outputs`
    /// :param deadline_miss: Local deadline miss
    /// :type deadline_miss: :class:`LocalDeadlineMiss`
    ///
    /// :rtype: :class:`Outputs`
    fn output_rule(
        &self,
        _context: &mut Context,
        _state: Py<PyAny>,
        _outputs: Outputs,
        _deadline_miss: LocalDeadlineMiss,
    ) -> PyResult<Outputs> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }

    /// The initialize method is called by the zenoh flow runtime.
    /// This method is called when starting the data flow graph.
    /// Any operator has to implement this method.
    /// This method is use to initialize any state that can be useful
    /// for the operator (e.g. configuration parameters)
    /// It should then return the state to the runtime.
    ///
    /// :param configuration: Configuration
    /// :type configuraion: dict
    ///
    /// :rtype: any
    fn initialize(&self, _configuration: Option<PyObject>) -> PyResult<Py<PyAny>> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }

    /// The finalize method is called by the zenoh flow runtime.
    /// This method is called when stopping the data flow graph.
    /// Any operator has to implement this method.
    /// This method is use to finalize any state that can be useful
    /// for the operator (e.g. configuration)
    /// It should destroy the state.
    ///
    ///  :param state: Sink state
    /// :type state: any
    fn finalize(&self, _state: Py<PyAny>) -> PyResult<()> {
        Err(pyo3::exceptions::PyNotImplementedError::new_err(
            "Not implemented",
        ))
    }
}

/// The zenoh flow Python API.
///
/// This API it's NOT meant to be used directly, instead your operators, sink
/// and sources have to implement the methods provided by the different classes.
/// A .py can contain only one graph node.
/// Each .py needs to contain a register function that takes no parameter
/// and returns the node.
///
/// def register():
///     return MyGraphNode
///
///
/// Below some examples for simple source, sink and operator.
///
/// Examples:
/// ~~~~~~~~
///
///
/// Source:
/// """"""""
///
/// from zenoh_flow import Inputs, Outputs, Source
/// import time
///
/// class MyState:
///     def __init__(self, configuration):
///         self.value = 0
///         if configuration['value'] is not None:
///             self.value = int(configuration['value'])
///
/// class MySrc(Source):
///    def initialize(self, configuration):
///         return MyState(configuration)
///     def finalize(self, state):
///         return None
///     def run(self, _ctx, state):
///         state.value += 1
///         time.sleep(1)
///         return int_to_bytes(state.value)
///
/// def int_to_bytes(x: int) -> bytes:
///     return x.to_bytes((x.bit_length() + 7) // 8, 'big')
/// def register():
///     return MySrc
/// """"""""
///
///
/// Sink:
/// """"""""
/// from zenoh_flow import Sink
///
/// class MySink(Sink):
///     def initialize(self, configuration):
///         return None
///     def finalize(self, state):
///         return None
///     def run(self, _ctx, _state, input):
///         print(f"Received {input}")
///
/// def register():
///     return MySink
///""""""""""
///
///
/// Operator:
/// """"""""
/// from zenoh_flow import Inputs, Operator, Outputs
/// class MyState:
///     def __init__(self):
///         self.value = 0
///     def inc(self):
///         self.value += 1
///     def mod_2(self):
///         return (self.value % 2)
///     def mod_3(self):
///         return (self.value % 3)
///
/// class MyOp(Operator):
///     def initialize(self, configuration):
///          return MyState()
///     def finalize(self, state):
///         return None
///     def input_rule(self, _ctx, state, tokens):
///         # Using input rules
///         state.inc()
///         token = tokens.get('Data')
///         if state.mod_2():
///             token.set_action_consume()
///             return True
///         elif state.mod_3():
///             token.set_action_keep()
///             return True
///         else:
///             token.set_action_drop()
///             return False
///
///     def output_rule(self, _ctx, _state, outputs, _deadline_miss):
///         return outputs
///
///     def run(self, _ctx, _state, inputs):
///         # Getting the inputs
///         data = inputs.get('Data').data
///
///         # Computing over the inputs
///         int_data = int_from_bytes(data)
///         int_data = int_data * 2
///         # Producing the outputs
///         outputs = {'Data' : int_to_bytes(int_data)}
///         return outputs
///
/// def int_to_bytes(x: int) -> bytes:
///     return x.to_bytes((x.bit_length() + 7) // 8, 'big')
///
/// def int_from_bytes(xbytes: bytes) -> int:
///     return int.from_bytes(xbytes, 'big')
///
/// def register():
///     return MyOp
/// """"""""""""
#[pymodule]
fn zenoh_flow(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Source>()?;
    m.add_class::<Sink>()?;
    m.add_class::<Operator>()?;
    m.add_class::<Inputs>()?;
    m.add_class::<Outputs>()?;
    m.add_class::<Context>()?;
    m.add_class::<LocalDeadlineMiss>()?;
    m.add_class::<InputToken>()?;
    m.add_class::<DataMessage>()?;

    Ok(())
}
