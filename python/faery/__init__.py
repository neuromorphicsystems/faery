import pathlib
import typing

import numpy

from .decoder import Decoder
from .stream import DVS_DTYPE as DVS_DTYPE
from .stream import Array as Array
from .stream import Stream as Stream
from .stream import TimestampOffset as TimestampOffset


def stream_from_file(
    path: typing.Union[str, pathlib.Path],
    stream_id: typing.Optional[int] = None,
) -> Stream:
    if isinstance(path, str):
        path = pathlib.Path(path)
    return Decoder(path=path, stream_id=stream_id)


def stream_from_array(events: numpy.ndarray) -> Stream:
    return Array(events=events)
