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
from zenoh_flow import DataReceiver
from typing import Dict, Any


class MySink(Sink):
    def finalize(self):
        return None

    def __init__(
        self, configuration: Dict[str, Any], inputs: Dict[str, DataReceiver]
    ) -> None:
        self.in_stream = inputs.get("Value", None)

    async def run(self):
        data_msg = await self.in_stream.recv()
        print(f"Received {int_from_bytes(data_msg.data)}")
        return None


def int_from_bytes(xbytes: bytes) -> int:
    return int.from_bytes(xbytes, "big")


def register():
    return MySink
