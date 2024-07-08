from __future__ import annotations

import collections.abc
import pathlib
import types
import typing

import numpy
import numpy.lib.recfunctions

from . import common
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


class DecoderIterator(stream.StreamIterator):
    def __init__(
        self,
        file_type: common.FileType,
        track_id: typing.Optional[int],
        is_atis: bool,
        dimensions: tuple[int, int],
        inner: collections.abc.Iterable,
    ):
        super().__init__()
        self.file_type = file_type
        self.track_id = track_id
        self.is_atis = is_atis
        self.dimensions = dimensions
        self.inner = iter(inner)

    def __next__(self) -> numpy.ndarray:
        assert self.inner is not None
        try:
            if self.file_type == common.FileType.AEDAT:
                while True:
                    track, packet = self.inner.__next__()
                    if (
                        track.id == self.track_id
                        and track.data_type == "events"
                        and len(packet) > 0
                    ):
                        return packet
            elif self.file_type == common.FileType.DAT:
                events: numpy.ndarray = self.inner.__next__()
                numpy.clip(events["payload"], 0, 1, events["payload"])
                return events.astype(
                    dtype=stream.DVS_DTYPE,
                    casting="unsafe",
                    copy=False,
                )
            elif self.file_type == common.FileType.ES:
                if self.is_atis:
                    while True:
                        atis_events: numpy.ndarray = self.inner.__next__()
                        mask = numpy.logical_not(atis_events["exposure"])
                        if len(mask) == 0:
                            continue
                        events = numpy.zeros(
                            numpy.count_nonzero(mask),
                            dtype=stream.DVS_DTYPE,
                        )
                        events["t"] = atis_events["t"][mask]
                        events["x"] = atis_events["x"][mask]
                        events["y"] = self.dimensions[1] - 1 - atis_events["y"][mask]
                        events["on"] = atis_events["polarity"][mask]
                        return events
                dvs_events: numpy.ndarray = self.inner.__next__()
                dvs_events["y"] = self.dimensions[1] - 1 - dvs_events["y"]
                return dvs_events
            elif self.file_type == common.FileType.EVT:
                while True:
                    packet = self.inner.__next__()
                    if "events" in packet:
                        return packet["events"]
            else:
                raise Exception(f"type {self.file_type} not implemented")
        except StopIteration as exception:
            raise exception

    def __exit__(
        self,
        exception_type: typing.Optional[typing.Type[BaseException]],
        value: typing.Optional[BaseException],
        traceback: typing.Optional[types.TracebackType],
    ) -> bool:
        if self.inner is None:
            return False
        result = self.inner.__exit__(exception_type, value, traceback)  # type: ignore
        self.inner = None
        return result

    def close(self):
        if self.inner is not None:
            self.inner.__exit__(None, None, None)  # type: ignore
            self.inner = None


class Decoder(stream.Stream):
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

    def __init__(
        self,
        path: typing.Union[pathlib.Path, str],
        track_id: typing.Optional[int] = None,
        dimensions_fallback: tuple[int, int] = (1280, 720),
        version_fallback: typing.Optional[
            typing.Literal["dat1", "dat2", "evt2", "evt2.1", "evt3"]
        ] = None,
        t0: common.Time = 0,
        file_type: typing.Optional[common.FileType] = None,
    ):
        super().__init__()
        self.path = pathlib.Path(path)
        self.track_id = track_id
        self.dimensions_fallback = dimensions_fallback
        self.version_fallback = version_fallback
        self.t0 = common.parse_timestamp(t0)
        self.file_type = (
            common.FileType.guess(self.path) if file_type is None else file_type
        )
        self.inner_dimensions: tuple[int, int]
        self.event_type: typing.Optional[str] = None
        self._time_range_us: typing.Optional[tuple[int, int]] = None
        if self.file_type == common.FileType.AEDAT:
            with aedat.Decoder(self.path) as decoder:
                found = False
                for track in decoder.tracks():
                    if self.track_id is None:
                        if track.data_type == "events":
                            self.track_id = track.id
                            assert track.dimensions is not None
                            self.inner_dimensions = track.dimensions
                            found = True
                            break
                    else:
                        if track.id == self.track_id:
                            if track.data_type != "events":
                                raise Exception(
                                    f'track {self.track_id} does not contain events (its type is "{track.data_type}")'
                                )
                            assert track.dimensions is not None
                            self.inner_dimensions = track.dimensions
                            found = True
                            break
                if not found:
                    if self.track_id is None:
                        raise Exception(f"the file contains no event tracks")
                    else:
                        raise Exception(
                            f"track {self.track_id} not found (the available ids are {[track.id for track in decoder.tracks()]})"
                        )
        elif self.file_type == common.FileType.DAT:
            if self.version_fallback is None:
                self.version_fallback = "dat2"
            with dat.Decoder(
                self.path,
                self.dimensions_fallback,
                self.version_fallback,  # type: ignore
            ) as decoder:
                self.event_type = decoder.event_type
                if self.event_type != "cd":
                    raise Exception(
                        f'the stream "{self.path}" has the unsupported type "{self.event_type}"'
                    )
                assert decoder.dimensions is not None
                self.inner_dimensions = decoder.dimensions
        elif self.file_type == common.FileType.ES:
            with event_stream.Decoder(
                path=self.path,
                t0=self.t0,
            ) as decoder:
                self.event_type = decoder.event_type
                if self.event_type != "dvs" and self.event_type != "atis":
                    raise Exception(
                        f'the stream "{self.path}" has the unsupported type "{self.event_type}"'
                    )
                assert decoder.dimensions is not None
                self.inner_dimensions = decoder.dimensions
        elif self.file_type == common.FileType.EVT:
            if self.version_fallback is None:
                self.version_fallback = "evt3"
            with evt.Decoder(
                self.path,
                self.dimensions_fallback,
                self.version_fallback,  # type: ignore
            ) as decoder:
                self.inner_dimensions = decoder.dimensions
        else:
            raise Exception(f"file type {self.file_type} not implemented")

    def dimensions(self) -> tuple[int, int]:
        return self.inner_dimensions

    def time_range_us(self) -> tuple[int, int]:
        if self._time_range_us is None:
            begin: typing.Optional[int] = None
            end: typing.Optional[int] = None
            for events in self:
                if len(events) > 0:
                    if begin is None:
                        begin = events["t"][0]
                    end = events["t"][-1]
            if begin is None or end is None:
                self._time_range_us = (0, 1)
            else:
                self._time_range_us = (int(begin), int(end) + 1)
        return self._time_range_us

    def __iter__(self) -> stream.StreamIterator:
        if self.file_type == common.FileType.AEDAT:
            inner = aedat.Decoder(self.path)
        elif self.file_type == common.FileType.DAT:
            inner = dat.Decoder(self.path, self.dimensions_fallback, self.version_fallback)  # type: ignore
        elif self.file_type == common.FileType.ES:
            inner = event_stream.Decoder(path=self.path, t0=self.t0)
        elif self.file_type == common.FileType.EVT:
            inner = evt.Decoder(self.path, self.dimensions_fallback, self.version_fallback)  # type: ignore
        else:
            raise Exception(f"file type {self.file_type} not implemented")
        return DecoderIterator(
            file_type=self.file_type,
            track_id=self.track_id,
            is_atis=self.event_type == "atis",
            dimensions=self.inner_dimensions,
            inner=inner,
        )
