from __future__ import annotations

import enum
import pathlib
import types
import typing

import numpy
import numpy.lib.recfunctions

from . import stream

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


class FileType(enum.Enum):
    AEDAT = 0
    DAT = 1
    ES = 2
    EVT = 3

    def magic(self) -> typing.Optional[bytes]:
        if self == FileType.AEDAT:
            return b"#!AER-DAT4.0\r\n"
        if self == FileType.DAT:
            return None
        elif self == FileType.ES:
            return b"Event Stream"
        elif self == FileType.EVT:
            return None
        else:
            raise Exception(f"magic is not implemented for {self}")

    def extensions(self) -> list[str]:
        if self == FileType.AEDAT:
            return [".aedat", ".aedat4"]
        if self == FileType.DAT:
            return [".dat"]
        elif self == FileType.ES:
            return [".es"]
        elif self == FileType.EVT:
            return [".evt", ".raw"]
        else:
            raise Exception(f"extensions is not implemented for {self}")

    @staticmethod
    def guess(path: pathlib.Path) -> FileType:
        longest_magic = max(
            0 if magic is None else len(magic)
            for magic in (file_type.magic() for file_type in FileType)
        )
        with open(path, "rb") as file:
            magic = file.read(longest_magic)
        for file_type in FileType:
            if file_type.magic() == magic:
                return file_type
        extension = path.suffix
        for file_type in FileType:
            if any(
                extension == type_extension for type_extension in file_type.extensions()
            ):
                return file_type
        raise Exception(f"unsupported file {path}")


class DecoderIterator(stream.StreamIterator):
    def __init__(
        self,
        file_type: FileType,
        stream_id: typing.Optional[int],
        is_atis: bool,
        width: int,
        height: int,
        inner: typing.Iterable,
    ):
        super().__init__()
        self.file_type = file_type
        self.stream_id = stream_id
        self.is_atis = is_atis
        self.width = width
        self.height = height
        self.inner = iter(inner)

    def __next__(self) -> numpy.ndarray:
        assert self.inner is not None
        try:
            if self.file_type == FileType.AEDAT:
                while True:
                    packet = self.inner.__next__()
                    if (
                        "events" in packet
                        and packet["stream_id"] == self.stream_id
                        and len(packet["events"]) > 0
                    ):
                        packet["events"]["y"] = self.height - 1 - packet["events"]["y"]
                        return packet["events"]
            elif self.file_type == FileType.DAT:
                events: numpy.ndarray = self.inner.__next__()
                numpy.clip(events["payload"], 0, 1, events["payload"])
                numpy.lib.recfunctions.rename_fields(events, {"payload": "on"})
                events["y"] = self.height - 1 - events["y"]
                return events.astype(
                    dtype=stream.DVS_DTYPE,
                    casting="unsafe",
                    copy=False,
                )
            elif self.file_type == FileType.ES:
                if self.is_atis:
                    while True:
                        atis_events = self.inner.__next__()
                        mask = numpy.logical_not(atis_events["exposure"])
                        if len(mask) == 0:
                            continue
                        events = numpy.zeros(
                            numpy.count_nonzero(mask),
                            dtype=stream.DVS_DTYPE,
                        )
                        events["t"] = atis_events["t"][mask]
                        events["x"] = atis_events["x"][mask]
                        events["y"] = atis_events["y"][mask]
                        events["on"] = atis_events["polarity"][mask]
                        return events
                return self.inner.__next__()
            elif self.file_type == FileType.EVT:
                while True:
                    packet = self.inner.__next__()
                    if "events" in packet:
                        packet["events"]["y"] = self.height - 1 - packet["events"]["y"]
                        return packet["events"]
            else:
                raise Exception(f"type {self.file_type} not implemented")
        except StopIteration as exception:
            self.close()
            raise exception

    def __exit__(
        self,
        exception_type: typing.Optional[typing.Type[BaseException]],
        value: typing.Optional[BaseException],
        traceback: typing.Optional[types.TracebackType],
    ) -> bool:
        assert self.inner is not None
        result = self.inner.__exit__(exception_type, value, traceback)  # type: ignore
        self.inner = None
        return result

    def close(self):
        if self.inner is not None:
            if self.file_type == FileType.ES:
                self.inner.__exit__(None, None, None)  # type: ignore
            self.inner = None


class Decoder(stream.Stream):
    """An event file decoder (supports .aedat4, .es, .raw, and .dat).

    stream_id is only used if the type is aedat. It selects a specific stream in the container.
    If left unspecified (None), the first event stream is selected.

    size_fallback is only used if the file type is EVT (.raw) or DAT and if the file's header
    does not specify the size.

    version_fallback is only used if the file type is EVT (.raw) or DAT and if the file's header
    does not specify the version.

    Args:
        path (pathlib.Path): Path of the input event file.
        stream_id (typing.Optional[int], optional): Stream ID, only used with aedat files. Defaults to None.
        size_fallback (tuple[int, int]], optional): Size fallback for EVT (.raw) and DAT files. Defaults to (1280, 720).
        version_fallback (str, optional): Version fallback for EVT (.raw) and DAT files. Defaults to "2" for DAT and "3" for EVT.
        file_type (typing.Optional[FileType], optional): Override the type determination algorithm. Defaults to None.
    """

    def __init__(
        self,
        path: pathlib.Path,
        stream_id: typing.Optional[int] = None,
        size_fallback: tuple[int, int] = (1280, 720),
        version_fallback: typing.Optional[str] = None,
        file_type: typing.Optional[FileType] = None,
    ):
        super().__init__()
        self.path = path
        self.stream_id = stream_id
        self.size_fallback = size_fallback
        self.version_fallback = version_fallback
        self.file_type = FileType.guess(self.path) if file_type is None else file_type
        self.inner_width: int
        self.inner_height: int
        self.event_type: typing.Optional[str] = None
        if self.file_type == FileType.AEDAT:
            with aedat.Decoder(self.path) as decoder:
                if self.stream_id is None:
                    for id, stream in decoder.id_to_stream().items():
                        if stream["type"] == "events":
                            self.stream_id = id
                            self.inner_width = stream["width"]  # type: ignore
                            self.inner_height = stream["height"]  # type: ignore
                            break
                    if self.stream_id is None:
                        raise Exception(f"the file {self.path} contains no events")
                else:
                    stream = decoder.id_to_stream()[self.stream_id]
                    assert stream["type"] == "events"
                    assert isinstance(stream["width"], int)
                    assert isinstance(stream["height"], int)
                    self.inner_width = stream["width"]
                    self.inner_height = stream["height"]
        elif self.file_type == FileType.DAT:
            if self.version_fallback is None:
                self.version_fallback = "DAT2"
            with dat.Decoder(
                self.path,
                self.size_fallback,
                self.version_fallback,  # type: ignore
            ) as decoder:
                self.event_type = decoder.event_type
                if self.event_type != "dvs":
                    raise Exception(
                        f'the stream "{self.path}" has the unsupported type "{self.event_type}"'
                    )
                self.inner_width = decoder.width
                self.inner_height = decoder.height
        elif self.file_type == FileType.ES:
            with event_stream.Decoder(self.path) as decoder:
                self.event_type = decoder.event_type
                if self.event_type != "dvs" and self.event_type != "atis":
                    raise Exception(
                        f'the stream "{self.path}" has the unsupported type "{self.event_type}"'
                    )
                assert decoder.width is not None
                assert decoder.height is not None
                self.inner_width = decoder.width
                self.inner_height = decoder.height
        elif self.file_type == FileType.EVT:
            if self.version_fallback is None:
                self.version_fallback = "EVT3"
            with evt.Decoder(
                self.path,
                self.size_fallback,
                self.version_fallback,  # type: ignore
            ) as decoder:
                self.inner_width = decoder.width
                self.inner_height = decoder.height
        else:
            raise Exception(f"file type {self.file_type} not implemented")

    def width(self) -> int:
        return self.inner_width

    def height(self) -> int:
        return self.inner_height

    def time_range(self) -> tuple[int, int]:
        begin: typing.Optional[int] = None
        end: typing.Optional[int] = None
        for events in self:
            if len(events) > 0:
                if begin is None:
                    begin = events["t"][0]
                end = events["t"][-1]
        if begin is None or end is None:
            return (0, 1)
        return (int(begin), int(end) + 1)

    def __iter__(self) -> stream.StreamIterator:
        if self.file_type == FileType.AEDAT:
            inner = aedat.Decoder(self.path)
        elif self.file_type == FileType.DAT:
            inner = dat.Decoder(self.path, self.size_fallback, self.version_fallback)  # type: ignore
        elif self.file_type == FileType.ES:
            inner = event_stream.Decoder(self.path)
        elif self.file_type == FileType.EVT:
            inner = evt.Decoder(self.path, self.size_fallback, self.version_fallback)  # type: ignore
        else:
            raise Exception(f"file type {self.file_type} not implemented")
        return DecoderIterator(
            file_type=self.file_type,
            stream_id=self.stream_id,
            is_atis=self.event_type == "atis",
            width=self.inner_width,
            height=self.inner_height,
            inner=inner,
        )
