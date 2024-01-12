from __future__ import annotations

import pathlib
import typing
import types

import numpy

class Decoder:
    width: int
    height: int

    def __init__(
        self,
        path: typing.Union[pathlib.Path, str],
        size_fallback: typing.Optional[tuple[int, int]] = None,
        version_fallback: typing.Optional[typing.Literal["EVT2", "EVT2.1", "EVT3"]] = None,
    ): ...
    def __enter__(self) -> Decoder: ...
    def __exit__(
        self,
        exception_type: typing.Optional[typing.Type[BaseException]],
        value: typing.Optional[BaseException],
        traceback: typing.Optional[types.TracebackType],
    ) -> bool: ...
    def __iter__(self) -> Decoder: ...
    def __next__(self) -> dict[str, numpy.ndarray]: ...
