from __future__ import annotations

import dataclasses
import enum
import pathlib
import types
import typing

import numpy

from . import common
from . import encoder
from . import frame

if typing.TYPE_CHECKING:
    from . import aedat  # type: ignore
    from . import dat  # type: ignore
    from . import event_stream  # type: ignore
    from . import evt  # type: ignore
    from . import filter
    from . import frame
else:
    from .faery import aedat
    from .faery import dat
    from .faery import event_stream
    from .faery import evt

DVS_DTYPE: numpy.dtype = numpy.dtype(
    [("t", "<u8"), ("x", "<u2"), ("y", "<u2"), (("p", "on"), "?")]
)


@dataclasses.dataclass
class Box:
    left: int
    right: int
    top: int
    bottom: int

    @classmethod
    def from_tuple(cls, box: tuple[int, int, int, int]) -> "Box":
        return cls(left=box[0], right=box[1], top=box[2], bottom=box[3])


class StreamIterator:
    def __iter__(self):
        return self

    def __next__(self) -> numpy.ndarray:
        raise NotImplementedError()

    def close(self):
        """
        Cleans up resources (such as files) used by this iterator.
        """
        pass


class Stream:
    def dimensions(self) -> tuple[int, int]:
        """
        Stream dimensions in pixels.

        Returns:
            tuple[int, int]: Width (left-right direction) and height (top-bottom direction) in pixels.
        """
        raise NotImplementedError()

    def time_range_us(self) -> tuple[int, int]:
        """
        Timestamp of the stream's start and end, in microseconds.

        Start is always smaller than or equal to the first event's timestamp.

        End is always strictly larger than the last event's timestamp.

        For instance, if the stream contains 3 events with timestamps `[2, 71, 828]`, the time range may be `(2, 829)`.
        It may also be wider, for instance `(0, 1000)`.

        Returns:
            tuple[int, int]: First and one-past-last timestamps in µs.
        """
        raise NotImplementedError()

    def time_range(self) -> tuple[str, str]:
        """
        Timecodes of the stream's start and end.

        For instance, if the stream contains 3 events with timestamps `[2, 71, 828]`,
        the time range may be `("00:00:00.000002", "00:00:00.000829")`.
        It may also be wider, for instance `("00:00:00.000000", "00:00:00.001000")`.

        Returns:
            tuple[int, int]: First and one-past-last timecodes.
        """
        start, end = self.time_range_us()
        return (common.timestamp_to_timecode(start), common.timestamp_to_timecode(end))

    def __iter__(self) -> StreamIterator:
        raise NotImplementedError()

    def __enter__(self) -> "Stream":
        return self

    def close(self):
        pass

    def __exit__(
        self,
        exception_type: typing.Optional[typing.Type[BaseException]],
        value: typing.Optional[BaseException],
        traceback: typing.Optional[types.TracebackType],
    ) -> bool:
        self.close()
        return False

    def remove_on_events(self) -> "filter.Map":
        from .filter import Map

        return Map(
            parent=self, function=lambda events: events[numpy.logical_not(events["on"])]
        )

    def remove_off_events(self) -> "filter.Map":
        from .filter import Map

        return Map(parent=self, function=lambda events: events[events["on"]])

    def time_slice(
        self,
        start: typing.Union[int, float, str],
        end: typing.Union[int, float, str],
        zero: bool = False,
    ) -> "filter.TimeSlice":
        from .filter import TimeSlice

        return TimeSlice(parent=self, start=start, end=end, zero=zero)

    def event_slice(self, start: int, end: int) -> "Stream":
        from .filter import EventSlice

        return EventSlice(parent=self, start=start, end=end)

    def crop(self, left: int, right: int, top: int, bottom: int) -> "Stream":
        from .filter import Crop

        return Crop(parent=self, left=left, right=right, top=top, bottom=bottom)

    def mask(self, array: numpy.ndarray):
        from .filter import Mask

        return Mask(parent=self, array=array)

    def transpose(
        self,
        action: typing.Literal[
            "flip_left_right",
            "flip_bottom_top",
            "rotate_90_counterclockwise",
            "rotate_180",
            "rotate_270_counterclockwise",
            "flip_up_diagonal",
            "flip_down_diagonal",
        ],
    ) -> "Stream":
        from .filter import Transpose

        return Transpose(parent=self, action=action)

    def to_array(self) -> numpy.ndarray:
        return numpy.concatenate(list(self))

    def save(
        self,
        path: typing.Union[pathlib.Path, str],
        version: typing.Optional[
            typing.Literal["dat1", "dat2", "evt2", "evt2.1", "evt3"]
        ] = None,
        zero_t0: bool = True,
        compression: typing.Optional[
            typing.Tuple[typing.Literal["lz4", "zstd"], int]
        ] = aedat.LZ4_DEFAULT,
        file_type: typing.Optional[common.FileType] = None,
    ) -> str:
        """Writes the stream to an event file (supports .aedat4, .es, .raw, and .dat).

        version is only used if the file type is EVT (.raw) or DAT.

        zero_t0 is only used if the file type is ES, EVT (.raw) or DAT.
        The original t0 is stored in the header of EVT and DAT files, and is discarded if the file type is ES.

        compression is only used if the file type is AEDAT.

        Args:
            stream: An iterable of event arrays (structured arrays with dtype faery.DVS_DTYPE).
            path: Path of the output event file.
            dimensions: Width and height of the sensor.
            version: Version for EVT (.raw) and DAT files. Defaults to "dat2" for DAT and "evt3" for EVT.
            zero_t0: Whether to normalize timestamps and write the offset in the header for EVT (.raw) and DAT files. Defaults to True.
            compression: Compression for aedat files. Defaults to ("lz4", 1).
            file_type: Override the type determination algorithm. Defaults to None.

        Returns:
            The original t0 as a timecode if the file type is ES, EVT (.raw) or DAT, and if `zero_t0` is true. 0 as a timecode otherwise.
            To reconstruct the original timestamps when decoding ES files with Faery, pass the returned value to `faery.stream_from_file`.
            EVT (.raw) and DAT files do not need this (t0 is written in their header), but it is returned here anyway for compatibility
            with software than do not support the t0 header field.
        """
        return encoder.save(
            stream=self,
            path=path,
            dimensions=self.dimensions(),
            version=version,
            zero_t0=zero_t0,
            compression=compression,
            file_type=file_type,
        )

    def render(self) -> "frame.Render":
        pass


class ArrayIterator(StreamIterator):
    def __init__(self, events: numpy.ndarray):
        super().__init__()
        self.events = events
        self.consumed = False

    def __next__(self) -> numpy.ndarray:
        if self.consumed:
            raise StopIteration()
        self.consumed = True
        return self.events

    def close(self):
        self.consumed = True


class Array(Stream):
    def __init__(self, events: numpy.ndarray, dimensions: tuple[int, int]):
        super().__init__()
        assert self.events.dtype == DVS_DTYPE
        self.events = events
        self.inner_dimensions = dimensions

    def __iter__(self) -> StreamIterator:
        return ArrayIterator(self.events.copy())

    def dimensions(self) -> tuple[int, int]:
        return self.inner_dimensions

    def time_range_us(self) -> tuple[int, int]:
        if len(self.events) == 0:
            return (0, 1)
        return (int(self.events["t"][0]), int(self.events["t"][-1]) + 1)
