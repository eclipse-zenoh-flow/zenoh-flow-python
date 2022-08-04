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

from zenoh_flow import DataReceiver
from typing import Dict, Any, Callable


class Sink(object):
    '''
        The class representing a Zenoh Flow sink
    '''

    def setup(self, configuration: Dict[str, Any], inputs: Dict[str, DataReceiver]) -> Callable[[], Any]:
        '''
            The run method is called by the Zenoh Flow runtime.
            Any sink has to implement this method.
            This method is expected to return a function that iterates over the inputs interacting
            with the external world.

            :param configuration: Configuration
            :type configuration: dict
            :param inputs: The input streams
            :type inputs: :class:`Dict[str, Receiver]`

            :rtype: Callable[[], Any]
        '''
        raise NotImplementedError("Please implement your own method, Sink is an interface")


    def finalize(self) -> None:
        '''
            The finalize method is called by the zenoh flow runtime.
            This method is called when stopping the data flow graph.
            Any sink has to implement this method.
            This method is use to finalize any state that can be useful
            for the sink (e.g. close files)
            It should destroy the state.
        '''
        raise NotImplementedError("Please implement your own method, Sink is an interface")