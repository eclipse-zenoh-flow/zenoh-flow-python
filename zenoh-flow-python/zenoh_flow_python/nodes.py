#
# Copyright © 2022 ZettaScale Technology
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

from zenoh_flow_python import Context, Inputs, Outputs
from typing import Dict, Any
from abc import ABC, abstractmethod


class Operator(ABC):
    """
    An `Operator` is a node performing transformation over the data it receives,
    outputting the end result to downstream node(s).

    An Operator hence possesses at least one Input and one Output.
    """

    @abstractmethod
    def __init__(
            self,
            context: Context,
            configuration: Dict[str, Any],
            inputs: Inputs,
            outputs: Outputs,
    ):
        """
        The constructor is called once by the Zenoh-Flow runtime when the data
        flow is loaded.
        """

    @abstractmethod
    async def iteration(self) -> None:
        """
        The `iteration` is called by the Zenoh-Flow runtime in a loop once the
        data flow is started.
        """

    @abstractmethod
    def finalize(self) -> None:
        """
        The `finalize` method is called by the Zenoh-Flow runtime before
        destroying the node (e.g., upon stopping the data flow graph).
        """


class Source(ABC):
    """
    A `Source` feeds data into a data flow.

    As such, a Source only possesses Output(s).
    """

    @abstractmethod
    def __init__(
            self,
            context: Context,
            configuration: Dict[str, Any],
            outputs: Outputs,
    ):
        """
        The constructor is called once by the Zenoh-Flow runtime when the data
        flow is loaded.
        """

    @abstractmethod
    async def iteration(self) -> None:
        """
        The `iteration` is called by the Zenoh-Flow runtime in a loop once the
        data flow is started.
        """

    @abstractmethod
    def finalize(self) -> None:
        """
        The `finalize` method is called by the Zenoh-Flow runtime before
        destroying the node (e.g., upon stopping the data flow graph).
        """


class Sink(ABC):
    """
    A `Sink` exposes the outcome of the data flow processing.

    As such, a Sink only possesses Input(s).
    """

    @abstractmethod
    def __init__(
            self,
            context: Context,
            configuration: Dict[str, Any],
            inputs: Inputs,
    ):
        """
        The constructor is called once by the Zenoh-Flow runtime when the data
        flow is loaded.
        """

    @abstractmethod
    async def iteration(self) -> None:
        """
        The `iteration` is called by the Zenoh-Flow runtime in a loop once the
        data flow is started.
        """

    @abstractmethod
    def finalize(self) -> None:
        """
        The `finalize` method is called by the Zenoh-Flow runtime before
        destroying the node (e.g., upon stopping the data flow graph).
        """
