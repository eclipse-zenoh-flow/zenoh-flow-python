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

from zenoh_flow import Inputs
from zenoh_flow.types import Context
from typing import Dict, Any


class Sink(object):
    """
    The class representing a Zenoh Flow sink.

    More information about Zenoh Flow operators can be found here:
    https://github.com/eclipse-zenoh/zenoh-flow/wiki/Sink-%28v0.4.0%29

    The `__init__` method is called by the zenoh flow runtime.

    It takes the following parameters:

    :param configuration: Configuration
    :type configuration: dict
    :param inputs: The input streams
    :type inputs: :class:`Inputs`
    """

    def __init__(
        self,
        context: Context,
        configuration: Dict[str, Any],
        inputs: Inputs,
    ):
        """
        The `__init__` method is called by the Zenoh Flow runtime.
        Any sink has to implement this method.
        This method is expected to initialize the sink.
        (E.g. storing relevant configuration and inputs)

        :param context: Zenoh Flow context
        :type configuration: context
        :param configuration: Configuration
        :type configuration: dict
        :param inputs: The input streams
        :type inputs: :class:`Inputs`

        :rtype: None
        """
        raise NotImplementedError(
            "Please implement your own method, Sink is an interface"
        )

    async def iteration(self) -> None:
        """
        The run method is called by the Zenoh Flow runtime, in a loop.
        It allows wait on the inputs and interacting with the external world.

        """
        raise NotImplementedError(
            "Please implement your own method, Sink is an interface"
        )

    def finalize(self) -> None:
        """
        The finalize method is called by the zenoh flow runtime before
        destroying the node (e.g., upon stopping the data flow graph).

        It must implement all the required steps to destroy your sink state.
        """
        raise NotImplementedError(
            "Please implement your own method, Sink is an interface"
        )
