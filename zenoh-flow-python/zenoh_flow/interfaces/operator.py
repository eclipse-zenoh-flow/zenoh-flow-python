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

from zenoh_flow import Sender, Receiver
from zenoh_flow.types import Context, InputToken, LocalDeadlineMiss, DataMessage
from typing import Dict, Optional, Any

class Operator(object):
    '''
        The class representing a Zenoh Flow operator
    '''
    def setup(self, configuration: Dict[str, Any], context: Context, inputs: Dict[str, Receiver], outputs: Dict[str, Sender]) -> Callable[..., ...]:
        """
            The run method is called by the zenoh flow runtime.
            This method is expected to return a function that iterates over the outputs producing data.
            Any operator has to implement this method.

            :param configuration: Configuration
            :type configuration: dict
            :param context: The Zenoh Flow context
            :type context: :class:`Context`
            :param inputs: The input streams
            :type inputs: :class:`Dict[str, Receiver]`
            :param outputs: The output streams
            :type outputs: :class:`Dict[str, Sender]`

            :rtype: Callable[..., ...]
        """
        raise NotImplementedError("Please implement your own method, Operator is an interface")

    def finalize(self) -> None:
        """
            The finalize method is called by the zenoh flow runtime.
            This method is called when stopping the data flow graph.
            Any operator has to implement this method.
            This method is use to finalize any state that can be useful
            for the operator (e.g. configuration)
            It should destroy the state.

        """
        raise NotImplementedError("Please implement your own method, Operator is an interface")