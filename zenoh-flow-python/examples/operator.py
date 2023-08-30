#
# Copyright (c) 2021 - 2023 ZettaScale Technology
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

from zenoh_flow.interfaces import Operator
from zenoh_flow import Inputs, Outputs
from zenoh_flow.types import Context
from typing import Dict, Any


class MyOperator(Operator):
    def __init__(
        self,
        context: Context,
        configuration: Dict[str, Any],
        inputs: Inputs,
        outputs: Outputs,
    ):
        print(f"Context: {context}")
        self.in_stream = inputs.take("Data", int, lambda x: int.from_bytes(x, "big"))
        self.output = outputs.take("Data", int, lambda x: x.to_bytes((x.bit_length() + 7) // 8, "big"))

    def finalize(self) -> None:
        return None

    async def iteration(self) -> None:
        # in order to wait on multiple input streams use:
        # https://docs.python.org/3/library/asyncio-task.html#asyncio.gather
        # or
        # https://docs.python.org/3/library/asyncio-task.html#asyncio.wait
        data_msg = await self.in_stream.recv()

        await self.output.send(data_msg.data)
        return None


def register():
    return MyOperator
