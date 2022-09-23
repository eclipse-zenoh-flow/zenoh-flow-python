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
    The class representing a Zenoh Flow operator.

    The `__init__` method is called by the zenoh flow runtime.

    It takes the following parameters:

    :param configuration: Configuration
    :type configuration: dict
    :param inputs: The input streams
    :type inputs: :class:`Dict[str, DataReceiver]`
    :param outputs: The output streams
    :type outputs: :class:`Dict[str, DataSender]`

    """

    def __init__(
        self,
        configuration: Dict[str, Any],
        inputs: Dict[str, DataReceiver],
        outputs: Dict[str, DataSender],
    ):
        """
        The `__init__` method is called by the zenoh flow runtime.
        This method is expected to initialize the operator.
        (E.g. storing relevant configuration, inputs and outputs)
        Any operator has to implement this method.

        :param configuration: Configuration
        :type configuration: dict
        :param inputs: The input streams
        :type inputs: :class:`Dict[str, DataReceiver]`
        :param outputs: The output streams
        :type outputs: :class:`Dict[str, DataSender]`

        :rtype: None
        """
        raise NotImplementedError(
            "Please implement your own method, Operator is an interface"
        )

    async def run(self) -> None:
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
        The finalize method is called by the zenoh flow runtime before destroying the node (e.g., upon stopping the data flow graph).

        It must implement all the required steps to destroy your operator state.

        """
        raise NotImplementedError(
            "Please implement your own method, Operator is an interface"
        )
