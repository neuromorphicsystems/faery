from __future__ import annotations

import pathlib
import typing

import numpy

from .decoder import Decoder
from .stream import DVS_DTYPE as DVS_DTYPE
from .stream import Array as Array
from .stream import Stream as Stream
from .stream import TimestampOffset as TimestampOffset

if typing.TYPE_CHECKING:
    from . import aedat  # type: ignore
    from . import dat  # type: ignore
    from . import event_stream  # type: ignore
    from . import evt  # type: ignore
else:
    from .faery import aedat
    from .faery import dat
    from .faery import event_stream
    from .faery import evt


def stream_from_file(
    path: typing.Union[str, pathlib.Path],
    stream_id: typing.Optional[int] = None,
    size_fallback: tuple[int, int] = (1280, 720),
    version_fallback: typing.Optional[str] = None,
    file_type: typing.Optional[decoder.FileType] = None,
) -> Stream:
    if isinstance(path, str):
        path = pathlib.Path(path)
    return Decoder(
        path=path,
        stream_id=stream_id,
        size_fallback=size_fallback,
        version_fallback=version_fallback,
        file_type=file_type,
    )


def stream_from_array(events: numpy.ndarray, width: int, height: int) -> Stream:
    return Array(events=events, width=width, height=height)
