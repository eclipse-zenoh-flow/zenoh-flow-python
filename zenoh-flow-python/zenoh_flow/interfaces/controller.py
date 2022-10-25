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


from zenoh_flow import Input, Output
from typing import Any, Dict


class Controller(object):
    """
    The class representing a Zenoh Flow controller, a controller is
    something acting as source and sink at the same time. Eg. a device driver.

    The `__init__` method is called by the zenoh flow runtime.

    It takes the following parameters:

    :param configuration: Configuration
    :type configuration: dict
    :param inputs: The input streams
    :type inputs: :class:`Dict[str, Receiver]`
    :param outputs: The output streams
    :type outputs: :class:`Dict[str, Sender]`
    """

    def __init__(
        self,
        configuration: Dict[str, Any],
        inputs: Dict[str, Input],
        outputs: Dict[str, Output],
    ):
        """
        The `__init__` method is called by the zenoh flow runtime.
        This method is expected to initialize the controller.
         (E.g. storing relevant configuration, inputs and outputs)
         Any controller has to implement this method.

         :param configuration: Configuration
         :type configuration: dict
         :param inputs: The input streams
         :type inputs: :class:`Dict[str, Receiver]`
         :param outputs: The output streams
         :type outputs: :class:`Dict[str, Sender]`

         :rtype: None
        """
        raise NotImplementedError(
            "Please implement your own method, Controller is an interface"
        )

    async def iteration(self) -> None:
        """
        The iteration method is called by the Zenoh Flow runtime, in a loop.
        This method is expected to gets data from the inputs,
        producing data and sends over the outputs
        """
        raise NotImplementedError(
            "Please implement your own method, Controller is an interface"
        )

    def finalize(self) -> None:
        """
        The finalize method is called by the zenoh flow runtime before
        destroying the node (e.g., upon stopping the data flow graph).

        It must implement all the required steps to destroy
        your controller state.
        """
        raise NotImplementedError(
            "Please implement your own method, Controller is an interface"
        )
