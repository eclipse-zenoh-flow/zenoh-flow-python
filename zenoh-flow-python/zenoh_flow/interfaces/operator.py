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

from zenoh_flow import DataReceiver, DataSender
from typing import Dict, Any


class Operator(object):
    """
    The class representing a Zenoh Flow operator
    """

    def __init__(
        self,
        configuration: Dict[str, Any],
        inputs: Dict[str, DataReceiver],
        outputs: Dict[str, DataSender],
    ) -> None:
        """
        The __init__ method is called by the zenoh flow runtime.
        This method is expected to initialize the operator.
        (E.g. storing relevant configuration, inputs and outputs)
        Any operator has to implement this method.

        :param configuration: Configuration
        :type configuration: dict
        :param inputs: The input streams
        :type inputs: :class:`Dict[str, Receiver]`
        :param outputs: The output streams
        :type outputs: :class:`Dict[str, Sender]`

        :rtype: None
        """
        raise NotImplementedError(
            "Please implement your own method, Operator is an interface"
        )

    async def run(self):
        """
        The run method is called by the Zenoh Flow runtime.
        This method is expected to iterates over the inputs for receiving data,
        producing data and sends over the outputs
        """
        raise NotImplementedError(
            "Please implement your own method, Operator is an interface"
        )

    def finalize(self) -> None:
        """
        The finalize method is called by the zenoh flow runtime.
        This method is called when stopping the data flow graph.
        Any operator has to implement this method.
        This method is use to finalize any state that can be useful
        for the operator (e.g. configuration)
        It should destroy the state.

        """
        raise NotImplementedError(
            "Please implement your own method, Operator is an interface"
        )
