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


from zenoh_flow import Sender
from zenoh_flow.types import Context
from typing import Any, Dict, Callable

class Source(object):
    '''
        The class representing a Zenoh Flow source
    '''
    def setup(self, configuration: Dict[str, Any], context: Context, outputs: Dict[str, Sender]) -> Callable[..., ...]:
        '''
            The setup method is called by the zenoh flow runtime.
            This method is expected to return a function that iterates over the outputs producing data.
            Any source has to implement this method.

            :param configuration: Configuration
            :type configuration: dict
            :param context: The Zenoh Flow context
            :type context: :class:`Context`
            :param outputs: The output streams
            :type outputs: :class:`Dict[str, Sender]`

            :rtype: Callable[..., ...]
        '''
        raise NotImplementedError("Please implement your own method, Source is an interface")

    def finalize(self) -> None:
        '''
            The finalize method is called by the zenoh flow runtime.
            This method is called when stopping the data flow graph.
            Any source has to implement this method.
            This method is use to finalize any state that can be useful
            for the source (e.g. close files)
            It should destroy the state.
        '''
        raise NotImplementedError("Please implement your own method, Source is an interface")
