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

from zenoh_flow.interfaces import Operator
from zenoh_flow import DataReceiver, DataSender
from typing import Dict, Any


class MyOp(Operator):
    def __init__(
        self,
        context: Context,
        configuration: Dict[str, Any],
        inputs: Dict[str, DataReceiver],
        outputs: Dict[str, DataSender],
    ):
        self.output = outputs.get("Data", None)
        self.in_stream = inputs.get("Data", None)
        self.cb_in = cb_in

    def finalize(self) -> None:
        return None

    async def run(self) -> None:
        # in order to wait on multiple input streams use:
        # https://docs.python.org/3/library/asyncio-task.html#asyncio.gather
        # or
        # https://docs.python.org/3/library/asyncio-task.html#asyncio.wait

        data_msg = await self.in_stream.recv()

        await self.output.send(data_msg.data)
        return None


def int_to_bytes(x: int) -> bytes:
    return x.to_bytes((x.bit_length() + 7) // 8, "big")


def int_from_bytes(xbytes: bytes) -> int:
    return int.from_bytes(xbytes, "big")


def register():
    return MyOp
