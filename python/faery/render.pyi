import collections.abc
import typing

import numpy
import numpy.typing

class RenderIterator:
    def __init__(
        self,
        parent: collections.abc.Iterable[numpy.ndarray],
        dimensions: tuple[int, int],
        next_frame_t: int,
        frame_duration: int,
        frame_count: int,
        decay: typing.Literal[
            "exponential",
            "linear",
            "step",
        ],
        tau: int,

    ): ...
    def __iter__(self) -> RenderIterator: ...
    def __next__(self) -> numpy.ndarray: ...
    def close(self): ...
