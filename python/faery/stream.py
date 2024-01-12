from __future__ import annotations

import dataclasses
import enum
import types
import typing

import numpy

DVS_DTYPE: numpy.dtype = numpy.dtype(
    [("t", "<u8"), ("x", "<u2"), ("y", "<u2"), ("on", "?")]
)


class TimestampOffset(enum.Enum):
    START = "start"
    FIRST = "first"


class Transpose(enum.Enum):
    FLIP_LEFT_RIGHT = "flip_left_right"
    FLIP_BOTTOM_TOP = "flip_bottom_top"
    ROTATE_90_COUNTERCLOCKWISE = "rotate_90_counterclockwise"
    ROTATE_180 = "rotate_180"
    ROTATE_270_COUNTERCLOCKWISE = "rotate_270_counterclockwise"
    FLIP_UP_DIAGONAL = "flip_up_diagonal"
    FLIP_DOWN_DIAGONAL = "flip_down_diagonal"


@dataclasses.dataclass
class Box:
    left: int
    bottom: int
    right: int
    top: int

    @classmethod
    def from_tuple(cls, box: tuple[int, int, int, int]) -> "Box":
        return cls(left=box[0], bottom=box[1], right=box[2], top=box[3])


class StreamIterator:
    def __iter__(self):
        return self

    def __next__(self) -> numpy.ndarray:
        raise NotImplementedError()

    def close(self):
        raise NotImplementedError()


class Stream:
    def width(self) -> int:
        raise NotImplementedError()

    def height(self) -> int:
        raise NotImplementedError()

    def time_range(self) -> tuple[int, int]:
        raise NotImplementedError()

    def __iter__(self) -> StreamIterator:
        raise NotImplementedError()

    def __enter__(self) -> "Stream":
        return self

    def __exit__(
        self,
        exception_type: typing.Optional[typing.Type[BaseException]],
        value: typing.Optional[BaseException],
        traceback: typing.Optional[types.TracebackType],
    ) -> bool:
        return False

    def time_slice(
        self,
        start: int,
        end: int,
        zero: typing.Optional[typing.Union[TimestampOffset, str]] = None,
    ) -> "Stream":
        # @TODO
        pass

    def event_slice(self, start: int, end: int, zero_first: bool = False) -> "Stream":
        # @TODO
        pass

    def crop(
        self,
        box: typing.Union[Box, tuple[int, int, int, int]],
        zero_left_bottom: bool = False,
    ) -> "Stream":
        if isinstance(box, tuple):
            box = Box.from_tuple(box)
        # @TODO
        pass

    def transpose(self, action: Transpose) -> "Stream":
        # @TODO
        pass

    def to_array(self) -> numpy.ndarray:
        return numpy.concatenate(list(self))


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
        pass


class Array(Stream):
    def __init__(self, events: numpy.ndarray, width: int, height: int):
        assert self.events.dtype == DVS_DTYPE
        self.events = events
        self.inner_width = width
        self.inner_height = height

    def __iter__(self) -> StreamIterator:
        return ArrayIterator(self.events.copy())

    def width(self) -> int:
        return self.inner_width

    def height(self) -> int:
        return self.inner_height

    def time_range(self) -> tuple[int, int]:
        if len(self.events) == 0:
            return (0, 1)
        return (int(self.events["t"][0]), int(self.events["t"][-1]) + 1)


class FilterIterator(StreamIterator):
    def __init__(self, parent: StreamIterator):
        super().__init__()
        self.parent = parent

    def close(self):
        self.parent.close()


class Filter(Stream):
    def __init__(self, parent: Stream):
        self.parent = parent

    def width(self) -> int:
        return self.parent.width()

    def height(self) -> int:
        return self.parent.height()

    def time_range(self) -> tuple[int, int]:
        return self.parent.time_range()
