from __future__ import annotations

import faery
import numpy

import assets


def validate(file: assets.File):
    # test format-specific decoders
    match file.format:
        case "aedat":
            print(f"faery.aedat.Decoder ({file.path.name})")
            assert file.dimensions is not None
            assert file.tracks is not None
            with faery.aedat.Decoder(file.path) as decoder:
                if file.header_lines is not None:
                    for index, line in enumerate(decoder.description().split("\n")):
                        assert (
                            file.header_lines[index] == line
                        ), f"{file=}, {index=}, {line=}, {file.header_lines[index]=}"
                tracks = decoder.tracks()
                assert len(tracks) == len(file.tracks)
                for track, file_track in zip(tracks, file.tracks):
                    assert track.id == file_track.id, f"{tracks=}, {file.tracks=}"
                    assert (
                        track.data_type == file_track.data_type
                    ), f"{tracks=}, {file.tracks=}"
                    assert (
                        track.dimensions == file_track.dimensions
                    ), f"{tracks=}, {file.tracks=}"
                field_to_hasher = file.field_to_hasher()
                for track, packet in decoder:
                    if track.data_type == "events":
                        assert isinstance(packet, numpy.ndarray)
                        field_to_hasher["t"].update(packet["t"].tobytes())
                        field_to_hasher["x"].update(packet["x"].tobytes())
                        field_to_hasher["y"].update(packet["y"].tobytes())
                        field_to_hasher["on"].update(packet["on"].tobytes())
                    elif track.data_type == "frame":
                        assert isinstance(packet, faery.aedat.Frame)
                        assert packet.pixels.shape == (
                            file.dimensions[1],
                            file.dimensions[0],
                        ), f"{packet.pixels.shape=}, {file.dimensions=}"
                        field_to_hasher["frame"].update(packet.pixels.tobytes())
                    elif track.data_type == "imus":
                        assert isinstance(packet, numpy.ndarray)
                        field_to_hasher["imus"].update(packet.tobytes())
                    elif track.data_type == "triggers":
                        assert isinstance(packet, numpy.ndarray)
                        field_to_hasher["triggers"].update(packet.tobytes())
                    else:
                        raise Exception(f'unexpected data type "{track.data_type}"')
                for field, hasher in field_to_hasher.items():
                    assert (
                        hasher.hexdigest() == file.field_to_digest[field]
                    ), f"{file=}, {field=}"
        case "dat2":
            print(f"faery.dat.Decoder ({file.path.name})")
            with faery.dat.Decoder(file.path) as decoder:
                assert decoder.version == "dat2"
                assert decoder.event_type == "cd"
                assert decoder.dimensions == file.dimensions
                field_to_hasher = file.field_to_hasher()
                for packet in decoder:
                    field_to_hasher["t"].update(packet["t"].tobytes())
                    field_to_hasher["x"].update(packet["x"].tobytes())
                    field_to_hasher["y"].update(packet["y"].tobytes())
                    field_to_hasher["on"].update(packet["payload"].tobytes())
                for field, hasher in field_to_hasher.items():
                    assert (
                        hasher.hexdigest() == file.field_to_digest[field]
                    ), f"{file=}, {field=}"
        case "es-atis":
            print(f"faery.event_stream.Decoder ({file.path.name})")
            assert file.t0 is not None
            with faery.event_stream.Decoder(
                path=file.path,
                t0=faery.parse_timestamp(file.t0),
            ) as decoder:
                assert decoder.version == "2.0.0"
                assert decoder.event_type == "atis"
                assert decoder.dimensions == file.dimensions
                assert decoder.dimensions is not None
                field_to_hasher = file.field_to_hasher(
                    [
                        "atis_t",
                        "atis_x",
                        "atis_y",
                        "atis_exposure",
                        "atis_polarity",
                        "atis_y_original",
                    ]
                )
                for packet in decoder:
                    field_to_hasher["atis_t"].update(packet["t"].tobytes())
                    field_to_hasher["atis_x"].update(packet["x"].tobytes())
                    field_to_hasher["atis_y"].update(
                        (decoder.dimensions[1] - 1 - packet["y"]).tobytes()
                    )
                    field_to_hasher["atis_exposure"].update(
                        packet["exposure"].tobytes()
                    )
                    field_to_hasher["atis_polarity"].update(
                        packet["polarity"].tobytes()
                    )
                    field_to_hasher["atis_y_original"].update(packet["y"].tobytes())
                for field, hasher in field_to_hasher.items():
                    assert (
                        hasher.hexdigest() == file.field_to_digest[field]
                    ), f"{file=}, {field=}"
        case "es-color":
            print(f"faery.event_stream.Decoder ({file.path.name})")
            assert file.t0 is not None
            with faery.event_stream.Decoder(
                path=file.path,
                t0=faery.parse_timestamp(file.t0),
            ) as decoder:
                assert decoder.version == "2.0.0"
                assert decoder.event_type == "color"
                assert decoder.dimensions == file.dimensions
                assert decoder.dimensions is not None
                field_to_hasher = file.field_to_hasher()
                for packet in decoder:
                    field_to_hasher["t"].update(packet["t"].tobytes())
                    field_to_hasher["x"].update(packet["x"].tobytes())
                    field_to_hasher["y"].update(
                        (decoder.dimensions[1] - 1 - packet["y"]).tobytes()
                    )
                    field_to_hasher["r"].update(packet["r"].tobytes())
                    field_to_hasher["g"].update(packet["g"].tobytes())
                    field_to_hasher["b"].update(packet["b"].tobytes())
                    field_to_hasher["y_original"].update(packet["y"].tobytes())
                for field, hasher in field_to_hasher.items():
                    assert (
                        hasher.hexdigest() == file.field_to_digest[field]
                    ), f"{file=}, {field=}"
        case "es-dvs":
            print(f"faery.event_stream.Decoder ({file.path.name})")
            assert file.t0 is not None
            with faery.event_stream.Decoder(
                path=file.path,
                t0=faery.parse_timestamp(file.t0),
            ) as decoder:
                assert decoder.version == "2.0.0"
                assert decoder.event_type == "dvs"
                assert decoder.dimensions == file.dimensions
                assert decoder.dimensions is not None
                field_to_hasher = file.field_to_hasher()
                for packet in decoder:
                    field_to_hasher["t"].update(packet["t"].tobytes())
                    field_to_hasher["x"].update(packet["x"].tobytes())
                    field_to_hasher["y"].update(
                        (decoder.dimensions[1] - 1 - packet["y"]).tobytes()
                    )
                    field_to_hasher["on"].update(packet["on"].tobytes())
                    if "y_original" in field_to_hasher:
                        field_to_hasher["y_original"].update(packet["y"].tobytes())
                for field, hasher in field_to_hasher.items():
                    assert (
                        hasher.hexdigest() == file.field_to_digest[field]
                    ), f"{file=}, {field=}"
        case "es-generic":
            print(f"faery.event_stream.Decoder ({file.path.name})")
            assert file.t0 is not None
            with faery.event_stream.Decoder(
                path=file.path,
                t0=faery.parse_timestamp(file.t0),
            ) as decoder:
                assert decoder.version == "2.0.0"
                assert decoder.event_type == "generic"
                assert decoder.dimensions == file.dimensions
                assert file.content_lines is not None
                field_to_hasher = file.field_to_hasher()
                index = 0
                first_t = None
                last_t = None
                for packet in decoder:
                    field_to_hasher["t"].update(packet["t"].tobytes())
                    for t, bytes in packet:
                        if first_t is None:
                            first_t = t
                        last_t = t
                        assert bytes == file.content_lines[index]
                        index += 1
                assert first_t is not None
                assert last_t is not None
                time_range = (
                    faery.timestamp_to_timecode(first_t),
                    faery.timestamp_to_timecode(last_t + 1),
                )
                assert (
                    time_range == file.time_range
                ), f"{time_range=}, {file.time_range=}"
                assert (
                    field_to_hasher["t"].hexdigest() == file.field_to_digest["t"]
                ), f'{file=}, field="t"'
        case "evt2":
            print(f"faery.evt.Decoder ({file.path.name})")
            assert file.dimensions is not None
            with faery.evt.Decoder(file.path, file.dimensions) as decoder:
                assert decoder.version == "evt2"
                assert decoder.dimensions == file.dimensions
                field_to_hasher = file.field_to_hasher()
                for packet in decoder:
                    if "events" in packet:
                        field_to_hasher["t"].update(packet["events"]["t"].tobytes())
                        field_to_hasher["x"].update(packet["events"]["x"].tobytes())
                        field_to_hasher["y"].update(packet["events"]["y"].tobytes())
                        field_to_hasher["on"].update(packet["events"]["on"].tobytes())
                for field, hasher in field_to_hasher.items():
                    assert (
                        hasher.hexdigest() == file.field_to_digest[field]
                    ), f"{file=}, {field=}"
        case "evt3":
            print(f"faery.evt.Decoder ({file.path.name})")
            with faery.evt.Decoder(file.path) as decoder:
                assert decoder.version == "evt3"
                assert decoder.dimensions == file.dimensions
                field_to_hasher = file.field_to_hasher()
                for packet in decoder:
                    if "events" in packet:
                        field_to_hasher["t"].update(packet["events"]["t"].tobytes())
                        field_to_hasher["x"].update(packet["events"]["x"].tobytes())
                        field_to_hasher["y"].update(packet["events"]["y"].tobytes())
                        field_to_hasher["on"].update(packet["events"]["on"].tobytes())
                for field, hasher in field_to_hasher.items():
                    assert (
                        hasher.hexdigest() == file.field_to_digest[field]
                    ), f"{file=}, {field=}"
        case _:
            raise Exception(f'unknown format "{file.format}"')

    # test the unified decoder (faery.stream_from_file) on supported formats
    if file.format in assets.DECODE_DVS_FORMATS:
        assert file.dimensions is not None
        print(f"faery.stream_from_file ({file.path.name})")
        if file.t0 is None:
            stream = faery.stream_from_file(
                file.path,
                dimensions_fallback=file.dimensions,
            )
        else:
            stream = faery.stream_from_file(
                file.path,
                dimensions_fallback=file.dimensions,
                t0=file.t0,
            )
        assert (
            stream.time_range() == file.time_range
        ), f"{stream.time_range()=}, {file.time_range=}"
        assert (
            stream.dimensions() == file.dimensions
        ), f"{stream.dimensions()=}, {file.dimensions=}"
        field_to_hasher = file.field_to_hasher(fields=["t", "x", "y", "on"])
        for events in stream:
            assert events.dtype == faery.DVS_DTYPE
            field_to_hasher["t"].update(events["t"].tobytes())
            field_to_hasher["x"].update(events["x"].tobytes())
            field_to_hasher["y"].update(events["y"].tobytes())
            field_to_hasher["on"].update(events["on"].tobytes())
        for field, hasher in field_to_hasher.items():
            assert (
                hasher.hexdigest() == file.field_to_digest[field]
            ), f"{file=}, {field=}"


if __name__ == "__main__":
    for file in assets.files:
        validate(file)
