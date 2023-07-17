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

from zenoh_flow.interfaces import Sink
from zenoh_flow import Inputs
from zenoh_flow.types import Context
from typing import Dict, Any


class MySink(Sink):
    def finalize(self):
        return None

    def __init__(
        self,
        context: Context,
        configuration: Dict[str, Any],
        inputs: Inputs,
    ):
        self.input = inputs.take("Value", int, lambda x: int.from_bytes(x, "big"))

    async def iteration(self) -> None:
        data_msg = await self.input.recv()
        print(f"Received {data_msg.data}")


def register():
    return MySink
