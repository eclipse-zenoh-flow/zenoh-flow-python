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

from zenoh_flow.interfaces import Source
from zenoh_flow import DataSender
from typing import Any, Dict
import time
import asyncio


class MySrc(Source):
    def __init__(self, configuration: Dict[str, Any], outputs: Dict[str, DataSender]):
        configuration = {} if configuration is None else configuration
        self.value = int(configuration.get("value", 0))
        self.output = outputs.get("Value", None)

    def finalize(self) -> None:
        return None

    async def run(self) -> None:
        await asyncio.sleep(0.5)
        self.value += 1
        print(f"Sending {self.value}")
        await self.output.send(int_to_bytes(self.value))


def int_to_bytes(x: int) -> bytes:
    return x.to_bytes((x.bit_length() + 7) // 8, "big")


def register():
    return MySrc
