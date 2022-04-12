#
# Copyright (c) 2022 ZettaScale Technology
#
# This program and the accompanying materials are made available under the
# terms of the Eclipse Public License 2.0 which is available at
# http://www.eclipse.org/legal/epl-2.0, or the Apache License, Version 2.0
# which is available at https://www.apache.org/licenses/LICENSE-2.0.
#
# SPDX-License-Identifier: EPL-2.0 OR Apache-2.0
#
# Contributors:
#   ZettaScale Zenoh Team, <zenoh@zettascale.tech>
#


'''
The Zenoh Flow Python API.


The *zenoh-flow-python* library provides a set of Python interfaces to write your
operators, source and sink for Zenoh Flow.

This API it's NOT meant to be used directly, instead your operators, sink
and sources have to implement the methods provided by th classes.
A .py file can contain only one graph node.

Each .py file needs to contain a register function that takes no parameter
and returns the node.

.. code-block:: python

    def register():
        return MyGraphNode


Below some examples for simple source, sink and operator.

Examples:
~~~~~~~~~

In the following you can find examples of soucres, sinks and operators.

Source:
"""""""
.. code-block:: python

    from zenoh_flow.interfaces import Source
    import time

    class MyState:
        def __init__(self, configuration):
            self.value = 0
            if configuration['value'] is not None:
                self.value = int(configuration['value'])

    class MySrc(Source):
        def initialize(self, configuration):
            return MyState(configuration)

        def finalize(self, state):
            return None

        def run(self, _ctx, state):
            state.value += 1
            time.sleep(1)
            return int_to_bytes(state.value)

    def int_to_bytes(x: int) -> bytes:
        return x.to_bytes((x.bit_length() + 7) // 8, 'big')

    def register():
        return MySrc



Sink:
"""""

.. code-block:: python

    from zenoh_flow.interfaces import Sink

    class MySink(Sink):
        def initialize(self, configuration):
            return None

        def finalize(self, state):
            return None

        def run(self, _ctx, _state, input):
            print(f"Received {input}")

    def register():
        return MySink



Operator:
"""""""""
.. code-block:: python

    from zenoh_flow.interfaces import Operator

    class MyState:
        def __init__(self):
            self.value = 0

        def inc(self):
            self.value += 1

        def mod_2(self):
            return (self.value % 2)

        def mod_3(self):
            return (self.value % 3)

    class MyOp(Operator):
        def initialize(self, configuration):
            return MyState()

        def finalize(self, state):
            return None

        def input_rule(self, _ctx, state, tokens):
            # Using input rules
            state.inc()
            token = tokens.get('Data')
            if state.mod_2():
                token.set_action_consume()
                return True
            elif state.mod_3():
                token.set_action_keep()
                return True
            else:
                token.set_action_drop()
                return False

        def output_rule(self, _ctx, _state, outputs, _deadline_miss = None):
            return outputs

        def run(self, _ctx, _state, inputs):
            # Getting the inputs
            data = inputs.get('Data').get_data()

            # Computing over the inputs
            int_data = int_from_bytes(data)
            int_data = int_data * 2
            # Producing the outputs
            outputs = {'Data' : int_to_bytes(int_data)}
            return outputs

    def int_to_bytes(x: int) -> bytes:
        return x.to_bytes((x.bit_length() + 7) // 8, 'big')

    def int_from_bytes(xbytes: bytes) -> int:
        return int.from_bytes(xbytes, 'big')

    def register():
        return MyOp

'''


from zenoh_flow import interfaces
from zenoh_flow import types


