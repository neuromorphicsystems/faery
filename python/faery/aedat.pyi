from __future__ import annotations

import pathlib
import typing
import types

import numpy

class Decoder:
    def __init__(self, path: typing.Union[pathlib.Path, str]): ...
    def __enter__(self) -> Decoder: ...
    def __exit__(
        self,
        exception_type: typing.Optional[typing.Type[BaseException]],
        value: typing.Optional[BaseException],
        traceback: typing.Optional[types.TracebackType],
    ) -> bool: ...
    def __iter__(self) -> Decoder: ...
    def __next__(self) -> dict[str, numpy.ndarray]: ...
    def id_to_stream(self) -> dict[int, dict[str, typing.Union[str, int]]]: ...
