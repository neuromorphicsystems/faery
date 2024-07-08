import collections.abc
import types
import typing

import numpy
import numpy.typing

from . import common
from . import stream


if typing.TYPE_CHECKING:
    from . import render  # type: ignore
else:
    from . import render


class FrameFloat64:
    """
    A frame with one channel per pixel, with values are in the range [-1, 1]
    """

    index: int
    timecode: str
    pixels: numpy.typing.NDArray[numpy.float64]


class FrameRgba8888:
    """
    A frame with 4 channels per pixels, with values in the range [0, 255]
    """

    index: int
    timecode: str
    pixels: numpy.typing.NDArray[numpy.uint8]


class FrameRgb888:
    """
    A frame with 3 channels per pixels, with values in the range [0, 255]
    """

    index: int
    timecode: str
    pixels: numpy.typing.NDArray[numpy.uint8]


class FrameStreamIteratorFloat64:
    def __iter__(self):
        return self

    def __next__(self) -> FrameFloat64:
        raise NotImplementedError()

    def close(self):
        """
        Cleans up resources (such as files) used by this iterator.
        """
        pass


class FrameStreamIteratorRgba8888:
    def __iter__(self):
        return self

    def __next__(self) -> FrameRgba8888:
        raise NotImplementedError()

    def close(self):
        """
        Cleans up resources (such as files) used by this iterator.
        """
        pass


class FrameStreamIteratorRgb888:
    def __iter__(self):
        return self

    def __next__(self) -> FrameRgb888:
        raise NotImplementedError()

    def close(self):
        """
        Cleans up resources (such as files) used by this iterator.
        """
        pass


class FrameStreamFloat64:
    def dimensions(self) -> tuple[int, int]:
        """
        Stream dimensions in pixels.

        Returns:
            tuple[int, int]: Width (left-right direction) and height (top-bottom direction) in pixels.
        """
        raise NotImplementedError()

    def frames_times(self) -> collections.abc.Iterable[str]:
        for timestamp in self.frames_times_us():
            yield common.timestamp_to_timecode(timestamp)

    def frames_times_us(self) -> collections.abc.Iterable[int]:
        raise NotImplementedError()

    def __iter__(self) -> FrameStreamIteratorFloat64:
        raise NotImplementedError()

    def __enter__(self) -> "FrameStreamFloat64":
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


class FrameStreamRgba8888:
    def dimensions(self) -> tuple[int, int]:
        """
        Stream dimensions in pixels.

        Returns:
            tuple[int, int]: Width (left-right direction) and height (top-bottom direction) in pixels.
        """
        raise NotImplementedError()

    def frames_times(self) -> collections.abc.Iterable[str]:
        for timestamp in self.frames_times_us():
            yield common.timestamp_to_timecode(timestamp)

    def frames_times_us(self) -> collections.abc.Iterable[int]:
        raise NotImplementedError()

    def __iter__(self) -> FrameStreamIteratorRgba8888:
        raise NotImplementedError()

    def __enter__(self) -> "FrameStreamRgba8888":
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


class FrameStreamRgb888:
    def dimensions(self) -> tuple[int, int]:
        """
        Stream dimensions in pixels.

        Returns:
            tuple[int, int]: Width (left-right direction) and height (top-bottom direction) in pixels.
        """
        raise NotImplementedError()

    def frames_times(self) -> collections.abc.Iterable[str]:
        for timestamp in self.frames_times_us():
            yield common.timestamp_to_timecode(timestamp)

    def frames_times_us(self) -> collections.abc.Iterable[int]:
        raise NotImplementedError()

    def __iter__(self) -> FrameStreamIteratorRgb888:
        raise NotImplementedError()

    def __enter__(self) -> "FrameStreamRgb888":
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


class Render(FrameStreamFloat64):
    def __init__(
        self,
        parent: stream.Stream,
        frame_duration: common.Time,
        decay: typing.Literal[
            "exponential",
            "linear",
            "step",
        ],
        tau: common.Time,
        ignore_polarity: bool,
    ):
        super().__init__()
        self.parent = parent
        self.frame_duration = common.parse_timestamp(frame_duration)
        self.decay = decay
        self.tau = common.parse_timestamp(tau)
        self.ignore_polarity = ignore_polarity

    def dimensions(self) -> tuple[int, int]:
        return self.parent.dimensions()

    def frames_times_us(self) -> collections.abc.Iterable[int]:
        time_range = self.parent.time_range_us()
        if time_range[1] - time_range[0] < self.frame_duration:
            return range(time_range[1], time_range[1] + 1)
        return range(
            time_range[0] + self.frame_duration, time_range[1] + 1, self.frame_duration
        )

    def __iter__(self) -> FrameStreamIteratorFloat64:
        iterator = render.RenderIterator(
            parent=self.parent.__iter__(),
            frame_duration=self.frame_duration,
            decay=self.decay,  # type: ignore
            tau=self.tau,
            ignore_polarity=self.ignore_polarity,
        )
        return iterator  # type: ignore
