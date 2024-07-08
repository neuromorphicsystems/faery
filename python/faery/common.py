import enum
import pathlib
import re
import typing

FULL_TIMECODE_PATTERN: re.Pattern = re.compile(r"^(\d+):(\d{2}):(\d{2})(\.\d{0,6})?$")
MINUTES_TIMECODE_PATTERN: re.Pattern = re.compile(r"^(\d+):(\d{2})(\.\d{0,6})?$")
SECONDS_TIMECODE_PATTERN: re.Pattern = re.compile(r"^(\d+)(\.\d{0,6})?$")


Time = typing.Union[int, float, str]
"""
A number of seconds encoded as an integer, a float, or a timecode.

A timecode is a string in the form "hh:mm:ss.µµµµµµ" where hh are hours, mm minutes, ss seconds and µµµµµµ microseconds.
Hours, minutes, and microseconds are optional.

See *tests/test_timestamps* for a list of example patterns.
"""


def parse_timestamp(value: Time) -> int:
    """
    Converts a timestamp (timecode or seconds) to an interger number of microseconds.

    Args:
        value: Timecode or seconds.

    Returns:
        int: Timestamp in microseconds.
    """
    if isinstance(value, int):
        assert value >= 0
        return value * 1000000
    if isinstance(value, float):
        assert value >= 0.0
        return round(value * 1e6)
    match = FULL_TIMECODE_PATTERN.match(value)
    if match is not None:
        result = (
            int(match[1]) * (1000000 * 60 * 60)
            + int(match[2]) * (1000000 * 60)
            + int(match[3]) * 1000000
        )
        if match[4] is not None:
            result += int(round(float(f"0{match[4]}") * 1000000.0))
        return result
    match = MINUTES_TIMECODE_PATTERN.match(value)
    if match is not None:
        result = int(match[1]) * (1000000 * 60) + int(match[2]) * 1000000
        if match[3] is not None:
            result += int(round(float(f"0{match[3]}") * 1000000.0))
        return result
    match = SECONDS_TIMECODE_PATTERN.match(value)
    if match is not None:
        result = int(match[1]) * 1000000
        if match[2] is not None:
            result += int(round(float(f"0{match[2]}") * 1000000.0))
        return result
    raise RuntimeError(f'parsing the timecode "{value}" failed')


def timestamp_to_timecode(value: int) -> str:
    value = int(value)
    hours = value // (1000000 * 60 * 60)
    value -= hours * (1000000 * 60 * 60)
    minutes = value // (1000000 * 60)
    value -= minutes * (1000000 * 60)
    seconds = value // 1000000
    value -= seconds * 1000000
    return f"{hours:>02}:{minutes:>02}:{seconds:>02}.{value:>06}"


def timestamp_to_seconds(value: int) -> float:
    return value * 1e6


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
    def guess(path: pathlib.Path) -> "FileType":
        longest_magic = max(
            0 if magic is None else len(magic)
            for magic in (file_type.magic() for file_type in FileType)
        )
        try:
            with open(path, "rb") as file:
                magic = file.read(longest_magic)
            for file_type in FileType:
                if file_type.magic() == magic:
                    return file_type
        except FileNotFoundError as exception:
            pass
        extension = path.suffix
        for file_type in FileType:
            if any(
                extension == type_extension for type_extension in file_type.extensions()
            ):
                return file_type
        raise Exception(f"unsupported file {path}")
