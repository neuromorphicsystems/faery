import enum
import pathlib
import types
import typing

import numpy

TYPE_ES: int = 0
TYPE_AEDAT: int = 1
TYPE_RAW: int = 2

from . import stream


class DecoderIterator(stream.StreamIterator):
    def __init__(self, type: int, stream_id: int, height: int, inner: typing.Iterable):
        super().__init__()
        self.type = type
        self.stream_id = stream_id
        self.height = height
        self.inner = iter(inner)

    def __next__(self) -> numpy.ndarray:
        assert self.inner is not None
        try:
            if self.type == TYPE_ES:
                return self.inner.__next__()
            elif self.type == TYPE_AEDAT:
                while True:
                    packet = self.inner.__next__()
                    if (
                        "events" in packet
                        and packet["stream_id"] == self.stream_id
                        and len(packet["events"]) > 0
                    ):
                        packet["events"]["y"] = self.height - 1 - packet["events"]["y"]
                        return packet["events"]
            elif self.type == TYPE_RAW:
                return self.inner.__next__()
            else:
                raise Exception(f"type {self.type} not implemented")
        except StopIteration as exception:
            self.close()
            raise exception

    def __exit__(
        self,
        exception_type: typing.Optional[typing.Type[BaseException]],
        value: typing.Optional[BaseException],
        traceback: typing.Optional[types.TracebackType],
    ) -> bool:
        assert self.decoder is not None
        if self.type == TYPE_ES:
            result = self.decoder.__exit__(exception_type, value, traceback)
        else:
            result = False
        self.decoder = None
        return result

    def close(self):
        if self.inner is not None:
            if self.type == TYPE_ES:
                self.inner.__exit__(None, None, None)  # type: ignore
            self.inner = None


class Decoder(stream.Stream):
    """An event file decoder (supports .aedat4, .es, .raw, and .dat).

    The stream_id selects a specific stream in an aedat container.
    This parameter is ignored if the file type is not aedat.
    The first event stream is used if the type is aedat and stream_id is None.

    Args:
        path (pathlib.Path): Path of the input event file.
        stream_id (typing.Optional[int], optional): Stream ID, only used with aedat files. Defaults to None.

    Raises:
        Exception: _description_
        Exception: _description_
        Exception: _description_
    """
    def __init__(self, path: pathlib.Path, stream_id: typing.Optional[int] = None):
        super().__init__()
        self.path = path
        self.stream_id = stream_id
        if path.suffix == ".es":
            self.type = TYPE_ES
        elif path.suffix == ".aedat4":
            self.type = TYPE_AEDAT
        else:
            with open(self.path, "rb") as file:
                magic = file.read(12)
            if magic == b"Event Stream":
                self.type = TYPE_ES
            elif magic == b"#!AER-DAT4.0":
                self.type = TYPE_AEDAT
            else:
                raise Exception(f"unsupported file {self.path}")
        self.inner_width: int
        self.inner_height: int
        if self.type == TYPE_ES:
            with event_stream.Decoder(self.path) as decoder:  # @TODO lower level
                assert decoder.type == "dvs"
                self.stream_width = decoder.width
                self.stream_height = decoder.height
        elif self.type == TYPE_AEDAT:
            with aedat.Decoder(self.path) as decoder:  # @TODO lower level
                found = False
                if stream_id is None:
                    for stream in decoder.id_to_stream().values():
                        if stream["type"] == "events":
                            self.stream_width = stream["width"]
                            self.stream_height = stream["height"]
                            found = True
                            break
                    if not found:
                        raise Exception(f"the file {self.path} contains no events")
                else:
                    stream = decoder.id_to_stream()[stream_id]
                    assert stream["type"] == "events"
                    self.stream_width = stream["width"]
                    self.stream_height = stream["height"]
        elif self.type == TYPE_RAW:
            pass  # @TODO
        else:
            raise Exception(f"type {self.type} not implemented")

    def width(self) -> int:
        return self.inner_width

    def height(self) -> int:
        return self.inner_height

    def __iter__(self) -> stream.StreamIterator:
        if self.type == TYPE_ES:
            inner = event_stream.Decoder(self.path)
        elif self.type == TYPE_AEDAT:
            inner = aedat.Decoder(self.path)
        elif self.type == TYPE_RAW:
            pass  # @TODO
        else:
            raise Exception(f"type {self.type} not implemented")
        return DecoderIterator(
            type=self.type,
            stream_id=self.stream_id,
            height=self.inner_height,
            inner=inner,
        )
