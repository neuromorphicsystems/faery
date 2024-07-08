import pathlib

import faery

# time slice
original_stream = faery.stream_from_file(
    pathlib.Path(__file__).resolve().parent / "data" / "dvs.es"
)
original_time_range = original_stream.time_range()
original_events = original_stream.to_array()
original_events_time_range = (
    faery.timestamp_to_timecode(original_events["t"][0]),
    faery.timestamp_to_timecode(original_events["t"][-1] + 1),
)
assert (
    original_time_range == original_events_time_range
), f"{original_time_range=}, {original_events_time_range=}"

sliced_stream = original_stream.time_slice(
    start="00:00:00.123456", end="00:00:00.678900"
)
sliced_time_range = sliced_stream.time_range()
sliced_events = sliced_stream.to_array()
sliced_events_time_range = (
    faery.timestamp_to_timecode(sliced_events["t"][0]),
    faery.timestamp_to_timecode(sliced_events["t"][-1] + 1),
)
assert sliced_time_range == ("00:00:00.123456", "00:00:00.678900")
assert sliced_events_time_range == ("00:00:00.124000", "00:00:00.678001")

sliced_stream = original_stream.time_slice(
    start="00:00:00.123456", end="00:00:00.678900", zero=True
)
sliced_time_range = sliced_stream.time_range()
sliced_events = sliced_stream.to_array()
sliced_events_time_range = (
    faery.timestamp_to_timecode(sliced_events["t"][0]),
    faery.timestamp_to_timecode(sliced_events["t"][-1] + 1),
)
assert sliced_time_range == ("00:00:00.000000", "00:00:00.555444")
assert sliced_events_time_range == ("00:00:00.000544", "00:00:00.554545")


# event slice
sliced_stream = original_stream.event_slice(start=100000, end=300000)
sliced_events = sliced_stream.to_array()
assert len(sliced_events) == 200000
