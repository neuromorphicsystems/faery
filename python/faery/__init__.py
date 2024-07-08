from __future__ import annotations

import pathlib
import typing

import numpy

from .common import FileType
from .common import Time as Time
from .common import parse_timestamp as parse_timestamp
from .common import timestamp_to_timecode as timestamp_to_timecode
from .common import timestamp_to_seconds as timestamp_to_seconds
from .decoder import Decoder
from .stream import DVS_DTYPE as DVS_DTYPE
from .stream import Array as Array
from .stream import Stream as Stream

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
    track_id: typing.Optional[int] = None,
    dimensions_fallback: tuple[int, int] = (1280, 720),
    version_fallback: typing.Optional[
        typing.Literal["dat1", "dat2", "evt2", "evt2.1", "evt3"]
    ] = None,
    t0: Time = 0,
    file_type: typing.Optional[FileType] = None,
) -> Stream:
    """An event file decoder (supports .aedat4, .es, .raw, and .dat).

    track_id is only used if the type is aedat. It selects a specific stream in the container.
    If left unspecified (None), the first event stream is selected.

    dimensions_fallback is only used if the file type is EVT (.raw) or DAT and if the file's header
    does not specify the size.

    version_fallback is only used if the file type is EVT (.raw) or DAT and if the file's header
    does not specify the version.

    t0 is only used if the file type is ES.

    Args:
        path: Path of the input event file.
        track_id: Stream ID, only used with aedat files. Defaults to None.
        dimensions_fallback: Size fallback for EVT (.raw) and DAT files. Defaults to (1280, 720).
        version_fallback: Version fallback for EVT (.raw) and DAT files. Defaults to "dat2" for DAT and "evt3" for EVT.
        t0: Initial time for ES files, in seconds. Defaults to None.
        file_type: Override the type determination algorithm. Defaults to None.
    """
    return Decoder(
        path=pathlib.Path(path),
        track_id=track_id,
        dimensions_fallback=dimensions_fallback,
        version_fallback=version_fallback,
        t0=t0,
        file_type=file_type,
    )


def stream_from_array(events: numpy.ndarray, dimensions: tuple[int, int]) -> Stream:
    return Array(events=events, dimensions=dimensions)
