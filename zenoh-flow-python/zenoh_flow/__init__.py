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


The *zenoh-flow-python* library provides a set of Python interfaces to write
your operators, source and sink for Zenoh Flow.

This API it's NOT meant to be used directly, instead your operators, sink
and sources have to implement the methods provided by th classes.
A .py file can contain only one graph node.

Each .py file needs to contain a register function that takes no parameter
and returns the node.

.. code-block:: python

    def register():
        return MyGraphNode

Each .py file is accompanied by a YAML file describing the node.

The Zenoh Flow installation guide is provided as part of the Wiki:
https://github.com/eclipse-zenoh/zenoh-flow/wiki/Installation-(v0.4.0)

Getting started is available here:
https://github.com/eclipse-zenoh/zenoh-flow/wiki/Getting-started-(v0.4.0)



Below some examples for simple source, sink and operator.

Examples:
~~~~~~~~~

In the following you can find examples of soucres, sinks and operators.

Source:
"""""""
.. code-block:: python

    from zenoh_flow.interfaces import Source
    from zenoh_flow import Outputs
    from zenoh_flow.types import Context
    from typing import Any, Dict
    import time
    import asyncio


    class MySrc(Source):
        def __init__(
            self,
            context: Context,
            configuration: Dict[str, Any],
            outputs: Outputs
        ):
            configuration = {} if configuration is None else configuration
            self.value = int(configuration.get("value", 0))
            self.output = outputs.take("Value", int, int_to_bytes)

        def finalize(self) -> None:
            return None

        async def iteration(self) -> None:
            await asyncio.sleep(0.5)
            self.value += 1
            print(f"Sending {self.value}")
            await self.output.send(self.value)


    def int_to_bytes(x: int) -> bytes:
        return x.to_bytes((x.bit_length() + 7) // 8, "big")


    def register():
    return MySrc



Sink:
"""""

.. code-block:: python

    from zenoh_flow.interfaces import Sink
    from zenoh_flow import Inputs
    from zenoh_flow.types import Context
    from typing import Dict, Any


    class MySink(Sink):

        def __init__(self,
            context: Context,
            configuration: Dict[str, Any],
            inputs: Inputs
        ):
            self.in_stream = inputs.take("Value", int, int_from_bytes)

        def finalize(self) -> None:
            return None



        async def iteration(self) -> None:
            data_msg = await self.in_stream.recv()
            print(f"Received {data_msg.get_data())}")
            return None


    def int_from_bytes(xbytes: bytes) -> int:
        return int.from_bytes(xbytes, "big")


    def register():
        return MySink



Operator:
"""""""""
.. code-block:: python

    from zenoh_flow.interfaces import Operator
    from zenoh_flow import Inputs, Outputs
    from zenoh_flow.types import Context
    from typing import Dict, Any


    class MyOp(Operator):
        def __init__(
            self,
            context: Context,
            configuration: Dict[str, Any],
            inputs: Inputs,
            outputs: Outputs,
        ):
            self.output = outputs.take("Data", int, int_to_bytes)
            self.in_stream = inputs.take("Data", int, int_from_bytes)

        def finalize(self) -> None:
            return None

        async def iteration(self) -> None:
            # in order to wait on multiple input streams use:
            # https://docs.python.org/3/library/asyncio-task.html#asyncio.gather
            # or
            # https://docs.python.org/3/library/asyncio-task.html#asyncio.wait

            data_msg = await self.in_stream.recv()
            new_data = data_msg.get_data() * 2
            await self.output.send(new_data)
            return None


    def int_to_bytes(x: int) -> bytes:
        return x.to_bytes((x.bit_length() + 7) // 8, "big")


    def int_from_bytes(xbytes: bytes) -> int:
        return int.from_bytes(xbytes, "big")


    def register():
        return MyOp


'''


from .zenoh_flow import InnerInput, InnerOutput, InnerDataMessage

from zenoh_flow import interfaces
from zenoh_flow import types
from zenoh_flow.types import Inputs, Outputs, Input, Output, DataMessage
