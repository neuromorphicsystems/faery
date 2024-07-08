import collections.abc
import typing

import numpy

from . import common
from . import stream


class FilterIterator(stream.StreamIterator):
    def __init__(self, parent: stream.StreamIterator):
        super().__init__()
        self.parent = parent

    def close(self):
        """
        Two types of signals are used to indicate the end of iteration.
        - Down the pipeline, StopIteration exceptions raised upon calling `self.parent.__next__`.
        - Up the pipeline, calls to `close` that indicate that a downstream filter terminated early.

        Filters that do not need to clean up resources do not need to override close and should define
        `__next__` as follows.
        ```
        def __next__(self) -> numpy.ndarray:
            events = self.parent.__next__()
            # perform operations on events
            return events
        ```

        Filters that need to clean up resources need to override `close` (to cleanup said resources)
        and must define `__next__` as follows.
        ```
        def __next__(self) -> numpy.ndarray:
            try:
                events = self.parent.__next__()
                # perform operations on events
                return events
            except StopIteration as exception:
                self.close()
                raise exception
        ```
        """
        self.parent.close()


class Filter(stream.Stream):
    def __init__(self, parent: stream.Stream):
        super().__init__()
        self.parent = parent

    def dimensions(self) -> tuple[int, int]:
        return self.parent.dimensions()

    def time_range_us(self) -> tuple[int, int]:
        return self.parent.time_range_us()


class MapIterator(FilterIterator):
    def __init__(
        self,
        parent: stream.StreamIterator,
        function: collections.abc.Callable[[numpy.ndarray], numpy.ndarray],
    ):
        super().__init__(parent=parent)
        self.function = function

    def __next__(self) -> numpy.ndarray:
        while True:
            events = self.parent.__next__()
            if len(events) > 0:
                events = self.function(events)
                if len(events) > 0:
                    return events


class Map(Filter):
    def __init__(
        self,
        parent: stream.Stream,
        function: collections.abc.Callable[[numpy.ndarray], numpy.ndarray],
    ):
        super().__init__(parent=parent)
        self.function = function

    def __iter__(self) -> MapIterator:
        return MapIterator(
            self.parent.__iter__(),
            function=self.function,
        )


class TimeSliceIterator(FilterIterator):
    def __init__(
        self,
        parent: stream.StreamIterator,
        start: int,
        end: int,
        zero: bool,
    ):
        super().__init__(parent=parent)
        self.start = start
        self.end = end
        self.zero = zero

    def __next__(self) -> numpy.ndarray:
        while True:
            events = self.parent.__next__()
            if len(events) > 0:
                if events["t"][-1] < self.start:
                    continue
                if events["t"][0] >= self.end:
                    raise StopIteration()
                events = events[
                    numpy.logical_and(events["t"] >= self.start, events["t"] < self.end)
                ]
                if len(events) > 0:
                    if self.zero:
                        events["t"] -= self.start
                    return events


class TimeSlice(Filter):
    def __init__(
        self,
        parent: stream.Stream,
        start: common.Time,
        end: common.Time,
        zero: bool,
    ):
        super().__init__(parent=parent)
        self.start = common.parse_timestamp(start)
        self.end = common.parse_timestamp(end)
        assert self.start < self.end, f"{start=} must be strictly smaller than {end=}"
        self.zero = zero

    def time_range_us(self) -> tuple[int, int]:
        parent_time_range_us = self.parent.time_range_us()
        if self.zero:
            return (
                max(self.start, parent_time_range_us[0]) - self.start,
                min(self.end, parent_time_range_us[1]) - self.start,
            )
        else:
            return (
                max(self.start, parent_time_range_us[0]),
                min(self.end, parent_time_range_us[1]),
            )

    def __iter__(self) -> TimeSliceIterator:
        return TimeSliceIterator(
            self.parent.__iter__(),
            start=self.start,
            end=self.end,
            zero=self.zero,
        )


class EventSliceIterator(FilterIterator):
    def __init__(
        self,
        parent: stream.StreamIterator,
        start: int,
        end: int,
    ):
        super().__init__(parent=parent)
        self.start = start
        self.end = end
        self.index = 0

    def __next__(self) -> numpy.ndarray:
        while True:
            events = self.parent.__next__()
            if len(events) > 0:
                if self.index + len(events) <= self.start:
                    self.index += len(events)
                    continue
                if self.index >= self.end:
                    self.index += len(events)
                    raise StopIteration()
                if len(events) > 0:
                    length = len(events)
                    events = events[
                        max(self.start - self.index, 0) : min(
                            self.end - self.index, length
                        )
                    ]
                    self.index += length
                    if len(events) > 0:
                        return events


class EventSlice(Filter):
    def __init__(
        self,
        parent: stream.Stream,
        start: int,
        end: int,
    ):
        assert start < end, f"{start=} must be strictly smaller than {end=}"
        super().__init__(parent=parent)
        self.start = start
        self.end = end

    def __iter__(self) -> EventSliceIterator:
        return EventSliceIterator(
            self.parent.__iter__(),
            start=self.start,
            end=self.end,
        )


class CropIterator(FilterIterator):
    def __init__(
        self,
        parent: stream.StreamIterator,
        left: int,
        right: int,
        top: int,
        bottom: int,
    ):
        super().__init__(parent=parent)
        self.left = left
        self.right = right
        self.top = top
        self.bottom = bottom

    def __next__(self) -> numpy.ndarray:
        while True:
            events = self.parent.__next__()
            if len(events) > 0:
                events = events[
                    numpy.logical_and.reduce(
                        (
                            events["x"] >= self.left,
                            events["x"] < self.right,
                            events["y"] >= self.top,
                            events["y"] < self.bottom,
                        )
                    )
                ]
                if len(events) > 0:
                    events["x"] -= self.left
                    events["y"] -= self.top
                    return events


class Crop(Filter):
    def __init__(
        self,
        parent: stream.Stream,
        left: int,
        right: int,
        top: int,
        bottom: int,
    ):
        super().__init__(parent=parent)
        dimensions = parent.dimensions()
        assert left < right, f"{left=} must be strictly smaller than {right=}"
        assert left >= 0
        assert right <= dimensions[0]
        assert top < bottom, f"{top=} must be strictly smaller than {bottom=}"
        assert top >= 0
        assert bottom <= dimensions[1]
        self.left = left
        self.right = right
        self.top = top
        self.bottom = bottom

    def dimensions(self) -> tuple[int, int]:
        return (self.right - self.left, self.bottom - self.top)

    def __iter__(self) -> CropIterator:
        return CropIterator(
            self.parent.__iter__(),
            left=self.left,
            right=self.right,
            top=self.top,
            bottom=self.bottom,
        )


class MaskIterator(FilterIterator):
    def __init__(
        self,
        parent: stream.StreamIterator,
        array: numpy.ndarray,
    ):
        super().__init__(parent=parent)
        self.array = array

    def __next__(self) -> numpy.ndarray:
        while True:
            events = self.parent.__next__()
            if len(events) > 0:
                events = events[self.array[events["y"], events["x"]]]
                if len(events) > 0:
                    return events


class Mask(Filter):
    def __init__(self, parent: stream.Stream, array: numpy.ndarray):
        super().__init__(parent=parent)
        assert array.dtype == numpy.dtype("?")
        assert len(array.shape) == 2
        dimensions = self.dimensions()
        if array.shape[0] != dimensions[1] or array.shape[1] != dimensions[0]:
            raise Exception(
                "array must be {}x{} (got {}x{})",
                dimensions[1],
                dimensions[0],
                array.shape[0],
                array.shape[1],
            )
        self.array = array

    def __iter__(self) -> MaskIterator:
        return MaskIterator(
            self.parent.__iter__(),
            array=self.array,
        )


class TransposeIterator(FilterIterator):
    def __init__(
        self,
        parent: stream.StreamIterator,
        action: typing.Literal[
            "flip_left_right",
            "flip_bottom_top",
            "rotate_90_counterclockwise",
            "rotate_180",
            "rotate_270_counterclockwise",
            "flip_up_diagonal",
            "flip_down_diagonal",
        ],
        dimensions: tuple[int, int],
    ):
        super().__init__(parent=parent)
        self.action = action
        self.dimensions = dimensions

    def __next__(self) -> numpy.ndarray:
        while True:
            events = self.parent.__next__()
            if len(events) > 0:
                if self.action == "flip_left_right":
                    events["x"] = self.dimensions[0] - 1 - events["x"]
                elif self.action == "flip_bottom_top":
                    events["y"] = self.dimensions[1] - 1 - events["y"]
                elif self.action == "rotate_90_counterclockwise":
                    x = events["x"].copy()
                    events["x"] = self.dimensions[0] - 1 - events["y"]
                    events["y"] = x
                elif self.action == "rotate_180":
                    events["x"] = self.dimensions[0] - 1 - events["x"]
                    events["y"] = self.dimensions[1] - 1 - events["y"]
                elif self.action == "rotate_270_counterclockwise":
                    x = events["x"].copy()
                    events["x"] = events["y"]
                    events["y"] = self.dimensions[1] - 1 - x
                elif self.action == "flip_up_diagonal":
                    x = events["x"].copy()
                    events["x"] = events["y"]
                    events["y"] = x
                elif self.action == "flip_down_diagonal":
                    x = events["x"].copy()
                    events["x"] = self.dimensions[0] - 1 - events["y"]
                    events["y"] = self.dimensions[1] - 1 - x
                else:
                    raise Exception(f'unknown action "{self.action}"')


class Transpose(Filter):
    def __init__(
        self,
        parent: stream.Stream,
        action: typing.Literal[
            "flip_left_right",
            "flip_bottom_top",
            "rotate_90_counterclockwise",
            "rotate_180",
            "rotate_270_counterclockwise",
            "flip_up_diagonal",
            "flip_down_diagonal",
        ],
    ):
        super().__init__(parent=parent)
        self.action = action

    def dimensions(self) -> tuple[int, int]:
        dimensions = self.parent.dimensions()
        if self.action in ("flip_left_right", "flip_bottom_top", "rotate_180"):
            return dimensions
        if self.action in (
            "rotate_90_counterclockwise",
            "rotate_270_counterclockwise",
            "flip_up_diagonal",
            "flip_down_diagonal",
        ):
            return (dimensions[1], dimensions[0])
        raise Exception(f'unknown action "{self.action}"')

    def __iter__(self) -> TransposeIterator:
        return TransposeIterator(
            self.parent.__iter__(),
            action=self.action,  # type: ignore
            dimensions=self.parent.dimensions(),
        )
