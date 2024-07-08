import pathlib
import shutil
import time

import faery

import assets
import test_decoders

data_generated = pathlib.Path(__file__).resolve().parent / "data_generated"
if data_generated.is_dir():
    shutil.rmtree(data_generated)
data_generated.mkdir()

# re-encode each format in its own type and check the generated file
for file in assets.files:
    output = data_generated / file.path.name
    match file.format:
        case "aedat":
            print(f"faery.aedat.Decoder + faery.aedat.Encoder ({file.path.name})")
            with faery.aedat.Decoder(file.path) as decoder:
                with faery.aedat.Encoder(
                    path=output,
                    description_or_tracks=decoder.description(),
                    compression=faery.aedat.LZ4_HIGHEST,
                ) as encoder:
                    for track, packet in decoder:
                        encoder.write(track.id, packet)
        case "dat2":
            print(f"faery.dat.Decoder + faery.dat.Encoder ({file.path.name})")
            with faery.dat.Decoder(file.path) as decoder:
                assert decoder.dimensions is not None
                with faery.dat.Encoder(
                    path=output,
                    version=decoder.version,
                    event_type="cd",
                    zero_t0=True,
                    dimensions=decoder.dimensions,
                ) as encoder:
                    for packet in decoder:
                        encoder.write(packet)
        case "es-atis":
            print(
                f"faery.event_stream.Decoder + faery.event_stream.Encoder ({file.path.name})"
            )
            with faery.event_stream.Decoder(
                path=file.path,
                t0=0,
            ) as decoder:
                assert decoder.dimensions is not None
                with faery.event_stream.Encoder(
                    path=output,
                    event_type="atis",
                    zero_t0=True,
                    dimensions=decoder.dimensions,
                ) as encoder:
                    for packet in decoder:
                        encoder.write(packet)
        case "es-color":
            print(
                f"faery.event_stream.Decoder + faery.event_stream.Encoder ({file.path.name})"
            )
            with faery.event_stream.Decoder(
                path=file.path,
                t0=0,
            ) as decoder:
                assert decoder.dimensions is not None
                with faery.event_stream.Encoder(
                    path=output,
                    event_type="color",
                    zero_t0=True,
                    dimensions=decoder.dimensions,
                ) as encoder:
                    for packet in decoder:
                        encoder.write(packet)
        case "es-dvs":
            print(
                f"faery.event_stream.Decoder + faery.event_stream.Encoder ({file.path.name})"
            )
            with faery.event_stream.Decoder(
                path=file.path,
                t0=0,
            ) as decoder:
                assert decoder.dimensions is not None
                with faery.event_stream.Encoder(
                    path=output,
                    event_type="dvs",
                    zero_t0=True,
                    dimensions=decoder.dimensions,
                ) as encoder:
                    for packet in decoder:
                        encoder.write(packet)
        case "es-generic":
            print(
                f"faery.event_stream.Decoder + faery.event_stream.Encoder ({file.path.name})"
            )
            with faery.event_stream.Decoder(
                path=file.path,
                t0=0,
            ) as decoder:
                with faery.event_stream.Encoder(
                    path=output,
                    event_type="generic",
                    zero_t0=True,
                    dimensions=None,
                ) as encoder:
                    for packet in decoder:
                        encoder.write(packet)
        case "evt2":
            print(f"faery.evt.Decoder + faery.evt.Encoder ({file.path.name})")
            with faery.evt.Decoder(file.path, file.dimensions) as decoder:
                with faery.evt.Encoder(
                    path=output,
                    version="evt2",
                    zero_t0=True,
                    dimensions=decoder.dimensions,
                ) as encoder:
                    for packet in decoder:
                        encoder.write(packet)
        case "evt3":
            print(f"faery.evt.Decoder + faery.evt.Encoder ({file.path.name})")
            with faery.evt.Decoder(file.path) as decoder:
                with faery.evt.Encoder(
                    path=output,
                    version="evt3",
                    zero_t0=True,
                    dimensions=decoder.dimensions,
                ) as encoder:
                    for packet in decoder:
                        encoder.write(packet)
        case _:
            raise Exception(f'unknown format "{file.format}"')
    test_decoders.validate(
        file.clone_with(
            path=output,
            format=file.format,
            field_to_digest=file.field_to_digest,
            header_lines=file.header_lines,
            tracks=file.tracks,
            content_lines=file.content_lines,
            t0=file.t0,
        )
    )

# test aedat compression modes
for file in assets.files:
    output = data_generated / file.path.name
    if file.format == "aedat":
        for compression, level in (
            faery.aedat.LZ4_FASTEST,
            faery.aedat.LZ4_DEFAULT,
            faery.aedat.LZ4_HIGHEST,
            faery.aedat.ZSTD_FASTEST,
            faery.aedat.ZSTD_DEFAULT,
            (
                "zstd",
                6,
            ),  # faery.aedat.ZSTD_HIGHEST = ("zstd", 22) is too slow for tests
        ):
            output = (
                data_generated
                / f"{file.path.stem}_{compression}_{level}{file.path.suffix}"
            )
            print(
                f"faery.aedat.Decoder + faery.aedat.Encoder, {compression}@{level} ({file.path.name})"
            )
            begin = time.monotonic()
            with faery.aedat.Decoder(file.path) as decoder:
                with faery.aedat.Encoder(
                    path=output,
                    description_or_tracks=decoder.tracks(),
                    compression=(compression, level),
                ) as encoder:
                    for track, packet in decoder:
                        encoder.write(track.id, packet)
            print(f"decoded + encoded in {time.monotonic() - begin:.3f} s")
            test_decoders.validate(
                file.clone_with(
                    path=output,
                    format=file.format,
                    field_to_digest=file.field_to_digest,
                    header_lines=None,
                    tracks=file.tracks,
                    content_lines=None,
                    t0=None,
                )
            )

# test the unified encoder
for file in assets.files:
    if file.format in assets.DECODE_DVS_FORMATS:
        assert file.dimensions is not None
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
        for output_format in assets.ENCODE_DVS_FORMATS:
            output = (
                data_generated
                / f"{file.path.stem}-as-{output_format}.{assets.format_to_extension(output_format)}"
            )
            print(f"faery.stream_from_file + save ({file.path.name} -> {output.name})")
            if output_format in ["dat2", "evt2", "evt3"]:
                version = output_format
            else:
                version = None
            t0 = stream.save(
                output,
                dimensions=file.dimensions,
                version=version,  # type: ignore
            )
            print(f"{t0=}") # @DEV
            test_decoders.validate(
                file.clone_with(
                    path=output,
                    format=output_format,
                    field_to_digest={
                        "t": file.field_to_digest["t"],
                        "x": file.field_to_digest["x"],
                        "y": file.field_to_digest["y"],
                        "on": file.field_to_digest["on"],
                    },
                    header_lines=None,
                    tracks=(
                        [
                            faery.aedat.Track(
                                id=0, data_type="events", dimensions=file.dimensions
                            )
                        ]
                        if output_format == "aedat"
                        else None
                    ),
                    content_lines=None,
                    t0=t0,
                )
            )
