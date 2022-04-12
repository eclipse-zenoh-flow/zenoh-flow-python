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



from zenoh_flow.types import Context
from typing import Any

class Source(object):
    '''
        The class representing a Zenoh Flow source
    '''
    def run(self, context: Context, state: Any) -> bytes:
        '''
            The run method is called by the zenoh flow runtime.
            This method is expected to produce data whenever it is called.
            Any source has to implement this method.

            :rtype: bytes
        '''
        raise NotImplementedError("Please implement your own method, Source is an interface")

    def initialize(self, configuration: dict) -> Any:
        '''
            The initialize method is called by the zenoh flow runtime.
            This method is called when starting the data flow graph.
            Any source has to implement this method.
            This method is use to initialize any state that can be useful
            for the source (e.g. open files)
            It should then return the state to the runtime.

            :param configuration: Configuration
            :type configuraion: dict

            :rtype: any
        '''
        raise NotImplementedError("Please implement your own method, Source is an interface")

    def finalize(self, state: Any) -> None:
        '''
            The finalize method is called by the zenoh flow runtime.
            This method is called when stopping the data flow graph.
            Any source has to implement this method.
            This method is use to finalize any state that can be useful
            for the source (e.g. close files)
            It should destroy the state.

            :param state: Source state
            :type state: any
        '''
        raise NotImplementedError("Please implement your own method, Source is an interface")