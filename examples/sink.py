from asyncio import *
from typing import Any, Dict
import logging
logging.getLogger().setLevel(0)

import sys
logging.debug(sys.path)

from zenoh_flow_python import *

class MySink(Sink):
    input: InputRaw

    def __init__(self, context: Context, configuration: Dict[str, Any], inputs: Inputs):
        logging.info(configuration["test"])
        self.input = inputs.take_raw("in")


    async def iteration(self) -> None:
        message = await self.input.recv_async()
        logging.info("timestamp: {}, payload: {}".format(message.timestamp(), message.payload()))


    def finalize(self) -> None:
        pass


def register():
    return MySink
