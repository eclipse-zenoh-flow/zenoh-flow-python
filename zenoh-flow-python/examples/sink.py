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
from typing import Dict, Any, Callable

class MySink(Sink):

    def finalize(self):
        return None

    def setup(self, configuration: Dict[str, Any], inputs: Dict[str, DataReceiver]) -> Callable[[], Any]:
        in_stream = inputs.get('Value', None)
        return lambda: run(in_stream)

async def run(in_stream):
        data_msg = await in_stream.recv()
        print(f"Received {int_from_bytes(data_msg.data)}")
        return None

def int_from_bytes(xbytes: bytes) -> int:
    return int.from_bytes(xbytes, 'big')

def register():
    # import asyncio
    # import threading
    # loop = asyncio.new_event_loop()
    # asyncio.set_event_loop(loop)

    # def run_loop(loop):
    #     loop.run_forever()
    # threading.Thread(target=run_loop, args=(loop,)).start()

    return MySink