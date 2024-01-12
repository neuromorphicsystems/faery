from __future__ import annotations

import dataclasses
import pathlib
import hashlib
import typing

import faery

dirname = pathlib.Path(__file__).resolve().parent


@dataclasses.dataclass(frozen=True)
class File:
    path: pathlib.Path
    field_to_digest: dict[str, str]
    width: typing.Optional[int]
    height: typing.Optional[int]
    time_range: tuple[int, int]


@dataclasses.dataclass(frozen=True)
class DvsFile(File):
    width: int
    height: int


name_to_file: dict[str, File] = {
    "atis.es": DvsFile(
        path=dirname / "tests" / "atis.es",
        field_to_digest={
            "atis_t": "74d540ff3fe7dc4b88c6580059feed586418095a8c0deb799927593c",
            "atis_x": "eb78132b1bee84c75243c37e1da72fe3e75b950b61b4b63ea677b153",
            "atis_y": "a6c8b1035f6c6be4919f95f97256593559ce23d6be9093017d196431",
            "atis_exposure": "40c5268b989d0f2d1b7bbf46b61075df0d7b902bc072abaa3d129840",
            "atis_polarity": "4907bd15c5aff7f3a54b9a82e45503e4bb77078cffe5b05a57542fdc",
            "t": "6f3f9af2e99d83707fd74fef3486abdd9f2f81680d3c4fcc306b1965",
            "x": "976eab4dc499e350481c3f568f71b2713e3ffa944bc1866f31448460",
            "y": "6aa82f1dfddd6948a7359816ad62f7fb2c59d00f803e4128868db87b",
            "on": "6f3c58f949e7c11c55707d11f739d98669882d551403c79482ab81f9",
        },
        width=320,
        height=240,
        time_range=(0, 999001),
    ),
    "color.es": File(
        path=dirname / "tests" / "color.es",
        field_to_digest={
            "t": "6f3f9af2e99d83707fd74fef3486abdd9f2f81680d3c4fcc306b1965",
            "x": "976eab4dc499e350481c3f568f71b2713e3ffa944bc1866f31448460",
            "y": "6aa82f1dfddd6948a7359816ad62f7fb2c59d00f803e4128868db87b",
            "r": "42a8fc278c02481a2dbb625643266dd4f8b5bfd90bb6b059014eb655",
            "g": "23bf9139d88ee294b760ca4c8b496c22201f7dba178c53e2c1ac5f97",
            "b": "960497899f172264075e4b77f6b704f9179443776de594aa4e31bd63",
        },
        width=320,
        height=240,
        time_range=(0, 999001),
    ),
    "davis346.aedat4": DvsFile(
        path=dirname / "tests" / "davis346.aedat4",
        field_to_digest={
            "t": "f1e093cad5afb6ecb971dfa2ef7646ab4ae0f467f73a48804e40bb68",
            "x": "1d8ea97b0febadfde24dd0b9e608682fc6934fc88656823b15f0e7a7",
            "y": "e13b60fcd0673a4d5a303d4ec4b68bca8008a65e59bd7949f6e8825c",
            "on": "6f99cf01187da8a05e1a032f3782de51b87e51bdc31356669bdd7cb9",
            "y_original": "18f89da35f8f10b24c3407b03aa7f82bdd7c8e6ab5369e2c30f8bad0",
            "frame": "6dbd0c0ea251788515bce54edf50b9f29d1995a0330a8b623504379b",
            "imus": "9dffb33769bdb00c67404c3a15479bbd7e204cdc7725976c2ec563ef",
            "triggers": "8479de279528d9d1a04b987ed95d54e2c641124cda618d7072ebc3b7",
        },
        width=346,
        height=260,
        time_range=(1589163147368868, 1589163149728814),
    ),
    "dvs.es": DvsFile(
        path=dirname / "tests" / "dvs.es",
        field_to_digest={
            "t": "6f3f9af2e99d83707fd74fef3486abdd9f2f81680d3c4fcc306b1965",
            "x": "976eab4dc499e350481c3f568f71b2713e3ffa944bc1866f31448460",
            "y": "6aa82f1dfddd6948a7359816ad62f7fb2c59d00f803e4128868db87b",
            "on": "6f3c58f949e7c11c55707d11f739d98669882d551403c79482ab81f9",
        },
        width=320,
        height=240,
        time_range=(0, 999001),
    ),
    "evt2.raw": DvsFile(
        path=dirname / "tests" / "evt2.raw",
        field_to_digest={
            "t": "9855ba39c1baea316ae623b72466e420df60a7510248391856eb4eaf",
            "x": "93fc6cff19b483fd48ca53345c6c1cf05123681d7b7272a1717e9303",
            "y": "ad9b57909232297bcc8832560b58c92175164c9dfeeeaa768d173911",
            "on": "d2b5e9c4047f4c431937b24cd46c8b504e055372b0ded1df77f0ffa1",
            "y_original": "07cfaef58e5fa71ce9910ed5b88fb7e9fcf71e4a8654cc2c4666f1ae",
        },
        width=640,
        height=480,
        time_range=(913716224, 913812096),
    ),
    "evt3.raw": DvsFile(
        path=dirname / "tests" / "evt3.raw",
        field_to_digest={
            "t": "5844f7335f33b60910c5956fac9a6b169414a54f2da8ca2c6d17012d",
            "x": "436762030fbc50f12caaf9fff1fa3a55d3aaff574e8d84159f9eabe6",
            "y": "4f60b41113e64a589214f6f9734999d0ef2ca211dc77269c33922807",
            "on": "7829ba15f633174ffae397128ce7b2c97bb39bcb7f0042060007c65a",
            "y_original": "e7793490007f5b8ad1aa61f3c5adb6af5471af1c40164cd44b17f961",
        },
        width=1280,
        height=720,
        time_range=(11200224, 21968222),
    ),
    "gen4.dat": DvsFile(
        path=dirname / "tests" / "gen4.dat",
        field_to_digest={
            "t": "78517458e03478cbd6830659dcb09393ba8e013793f2177f18840fc6",
            "x": "436762030fbc50f12caaf9fff1fa3a55d3aaff574e8d84159f9eabe6",
            "y": "4f60b41113e64a589214f6f9734999d0ef2ca211dc77269c33922807",
            "on": "7829ba15f633174ffae397128ce7b2c97bb39bcb7f0042060007c65a",
            "y_original": "e7793490007f5b8ad1aa61f3c5adb6af5471af1c40164cd44b17f961",
        },
        width=1280,
        height=720,
        time_range=(5856, 10773854),
    ),
    "generic.es": File(
        path=dirname / "tests" / "generic.es",
        field_to_digest={
            "t": "4be986c09dccc23887a40a261e8b95ac8c5ab8d0812efd0f78065d78"
        },
        width=None,
        height=None,
        time_range=(0, 1207923),
    ),
}

file = name_to_file["davis346.aedat4"]
with faery.aedat.Decoder(file.path) as decoder:
    assert len(decoder.id_to_stream().keys()) == 4
    assert decoder.id_to_stream()[0]["type"] == "events"
    assert decoder.id_to_stream()[0]["width"] == file.width
    assert decoder.id_to_stream()[0]["height"] == file.height
    assert decoder.id_to_stream()[1]["type"] == "frame"
    assert decoder.id_to_stream()[1]["width"] == file.width
    assert decoder.id_to_stream()[1]["height"] == file.height
    assert decoder.id_to_stream()[2]["type"] == "imus"
    assert decoder.id_to_stream()[3]["type"] == "triggers"
    field_to_hasher = {
        field: hashlib.sha3_224() for field in file.field_to_digest.keys()
    }
    for packet in decoder:
        if "events" in packet:
            events = packet["events"]
            field_to_hasher["t"].update(events["t"].tobytes())
            field_to_hasher["x"].update(events["x"].tobytes())
            field_to_hasher["y_original"].update(events["y"].tobytes())
            field_to_hasher["y"].update(
                (file.height - 1 - events["y"]).tobytes()  # type: ignore
            )
            field_to_hasher["on"].update(events["on"].tobytes())
        if "frame" in packet:
            field_to_hasher["frame"].update(packet["frame"]["pixels"].tobytes())
        if "imus" in packet:
            field_to_hasher["imus"].update(packet["imus"].tobytes())
        if "triggers" in packet:
            field_to_hasher["triggers"].update(packet["triggers"].tobytes())
    for field, hasher in field_to_hasher.items():
        assert hasher.hexdigest() == file.field_to_digest[field]

file = name_to_file["gen4.dat"]
with faery.dat.Decoder(file.path) as decoder:
    assert decoder.event_type == "dvs"
    assert decoder.width == file.width
    assert decoder.height == file.height
    field_to_hasher = {
        field: hashlib.sha3_224() for field in file.field_to_digest.keys()
    }
    for packet in decoder:
        field_to_hasher["t"].update(packet["t"].tobytes())
        field_to_hasher["x"].update(packet["x"].tobytes())
        field_to_hasher["y_original"].update(packet["y"].tobytes())
        field_to_hasher["y"].update((decoder.height - 1 - packet["y"]).tobytes())
        field_to_hasher["on"].update(packet["payload"].tobytes())
    for field, hasher in field_to_hasher.items():
        assert hasher.hexdigest() == file.field_to_digest[field]

file = name_to_file["generic.es"]
with faery.event_stream.Decoder(file.path) as decoder:
    assert decoder.event_type == "generic"
    assert decoder.width == file.width
    assert decoder.height == file.height
    t_hasher = hashlib.sha3_224()
    expected_bytes = [
        b"Lorem",
        b"ipsum",
        b"dolor",
        b"sit",
        b"amet,",
        b"consectetur",
        b"adipiscing",
        b"elit,",
        b"sed",
        b"do",
        b"eiusmod",
        b"tempor",
        b"incididunt",
        b"ut",
        b"labore",
        b"et",
        b"dolore",
        b"magna",
        b"aliqua.",
        b"Ut",
        b"enim",
        b"ad",
        b"minim",
        b"veniam,",
        b"quis",
        b"nostrud",
        b"exercitation",
        b"ullamco",
        b"laboris",
        b"nisi",
        b"ut",
        b"aliquip",
        b"ex",
        b"ea",
        b"commodo",
        b"consequat.",
        b"Duis",
        b"aute",
        b"irure",
        b"dolor",
        b"in",
        b"reprehenderit",
        b"in",
        b"voluptate",
        b"velit",
        b"esse",
        b"cillum",
        b"dolore",
        b"eu",
        b"fugiat",
        b"nulla",
        b"pariatur.",
        b"Excepteur",
        b"sint",
        b"occaecat",
        b"cupidatat",
        b"non",
        b"proident,",
        b"sunt",
        b"in",
        b"culpa",
        b"qui",
        b"officia",
        b"deserunt",
        b"mollit",
        b"anim",
        b"id",
        b"est",
        b"laborum.",
        b"Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.",
    ]
    index = 0
    for packet in decoder:
        t_hasher.update(packet["t"].tobytes())
        for _, bytes in packet:
            assert bytes == expected_bytes[index]
            index += 1
    assert t_hasher.hexdigest() == file.field_to_digest["t"]

file = name_to_file["dvs.es"]
with faery.event_stream.Decoder(file.path) as decoder:
    assert decoder.event_type == "dvs"
    assert decoder.width == file.width
    assert decoder.height == file.height
    field_to_hasher = {
        field: hashlib.sha3_224() for field in file.field_to_digest.keys()
    }
    for packet in decoder:
        field_to_hasher["t"].update(packet["t"].tobytes())
        field_to_hasher["x"].update(packet["x"].tobytes())
        field_to_hasher["y"].update(packet["y"].tobytes())
        field_to_hasher["on"].update(packet["on"].tobytes())
    for field, hasher in field_to_hasher.items():
        assert hasher.hexdigest() == file.field_to_digest[field]

file = name_to_file["atis.es"]
with faery.event_stream.Decoder(file.path) as decoder:
    assert decoder.event_type == "atis"
    assert decoder.width == file.width
    assert decoder.height == file.height
    field_to_hasher = {
        field: hashlib.sha3_224()
        for field in file.field_to_digest.keys()
        if field.startswith("atis_")
    }
    for packet in decoder:
        field_to_hasher["atis_t"].update(packet["t"].tobytes())
        field_to_hasher["atis_x"].update(packet["x"].tobytes())
        field_to_hasher["atis_y"].update(packet["y"].tobytes())
        field_to_hasher["atis_exposure"].update(packet["exposure"].tobytes())
        field_to_hasher["atis_polarity"].update(packet["polarity"].tobytes())
    for field, hasher in field_to_hasher.items():
        assert hasher.hexdigest() == file.field_to_digest[field]

file = name_to_file["color.es"]
with faery.event_stream.Decoder(file.path) as decoder:
    assert decoder.event_type == "color"
    assert decoder.width == 320
    assert decoder.height == 240
    field_to_hasher = {
        field: hashlib.sha3_224() for field in file.field_to_digest.keys()
    }
    for packet in decoder:
        field_to_hasher["t"].update(packet["t"].tobytes())
        field_to_hasher["x"].update(packet["x"].tobytes())
        field_to_hasher["y"].update(packet["y"].tobytes())
        field_to_hasher["r"].update(packet["r"].tobytes())
        field_to_hasher["g"].update(packet["g"].tobytes())
        field_to_hasher["b"].update(packet["b"].tobytes())
    for field, hasher in field_to_hasher.items():
        assert hasher.hexdigest() == file.field_to_digest[field]

file = name_to_file["evt2.raw"]
assert isinstance(file, DvsFile)
with faery.evt.Decoder(file.path, (file.width, file.height)) as decoder:
    assert decoder.width == file.width
    assert decoder.height == file.height
    field_to_hasher = {
        field: hashlib.sha3_224() for field in file.field_to_digest.keys()
    }
    for packet in decoder:
        if "events" in packet:
            field_to_hasher["t"].update(packet["events"]["t"].tobytes())
            field_to_hasher["x"].update(packet["events"]["x"].tobytes())
            field_to_hasher["y_original"].update(packet["events"]["y"].tobytes())
            field_to_hasher["y"].update(
                (decoder.height - 1 - packet["events"]["y"]).tobytes()
            )
            field_to_hasher["on"].update(packet["events"]["on"].tobytes())
    for field, hasher in field_to_hasher.items():
        assert hasher.hexdigest() == file.field_to_digest[field]

file = name_to_file["evt3.raw"]
with faery.evt.Decoder(file.path) as decoder:
    assert decoder.width == file.width
    assert decoder.height == file.height
    field_to_hasher = {
        field: hashlib.sha3_224() for field in file.field_to_digest.keys()
    }
    for packet in decoder:
        if "events" in packet:
            field_to_hasher["t"].update(packet["events"]["t"].tobytes())
            field_to_hasher["x"].update(packet["events"]["x"].tobytes())
            field_to_hasher["y_original"].update(packet["events"]["y"].tobytes())
            field_to_hasher["y"].update(
                (decoder.height - 1 - packet["events"]["y"]).tobytes()
            )
            field_to_hasher["on"].update(packet["events"]["on"].tobytes())
    for field, hasher in field_to_hasher.items():
        assert hasher.hexdigest() == file.field_to_digest[field]

for file in name_to_file.values():
    if isinstance(file, DvsFile):
        stream = faery.stream_from_file(
            file.path,
            size_fallback=(file.width, file.height),
        )
        assert stream.time_range() == file.time_range
        assert stream.width() == file.width
        assert stream.height() == file.height
        field_to_hasher = {field: hashlib.sha3_224() for field in ("t", "x", "y", "on")}
        for events in stream:
            field_to_hasher["t"].update(events["t"].tobytes())
            field_to_hasher["x"].update(events["x"].tobytes())
            field_to_hasher["y"].update(events["y"].tobytes())
            field_to_hasher["on"].update(events["on"].tobytes())
        for field, hasher in field_to_hasher.items():
            assert hasher.hexdigest() == file.field_to_digest[field]
