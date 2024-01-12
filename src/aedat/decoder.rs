use std::io::Read;

#[allow(
    dead_code,
    unused_imports,
    clippy::derivable_impls,
    clippy::derive_partial_eq_without_eq,
    clippy::extra_unused_lifetimes,
    clippy::size_of_in_element_count,
    clippy::needless_lifetimes,
    clippy::unnecessary_cast
)]
#[path = "./ioheader_generated.rs"]
pub(crate) mod ioheader_generated;

#[allow(
    dead_code,
    unused_imports,
    clippy::derivable_impls,
    clippy::derive_partial_eq_without_eq,
    clippy::extra_unused_lifetimes,
    clippy::size_of_in_element_count,
    clippy::needless_lifetimes,
    clippy::unnecessary_cast
)]
#[path = "./events_generated.rs"]
pub(crate) mod events_generated;

#[allow(
    dead_code,
    unused_imports,
    clippy::derivable_impls,
    clippy::derive_partial_eq_without_eq,
    clippy::extra_unused_lifetimes,
    clippy::size_of_in_element_count,
    clippy::needless_lifetimes,
    clippy::unnecessary_cast
)]
#[path = "./frame_generated.rs"]
pub(crate) mod frame_generated;

#[allow(
    dead_code,
    unused_imports,
    clippy::derivable_impls,
    clippy::derive_partial_eq_without_eq,
    clippy::extra_unused_lifetimes,
    clippy::size_of_in_element_count,
    clippy::needless_lifetimes,
    clippy::unnecessary_cast
)]
#[path = "./imus_generated.rs"]
pub(crate) mod imus_generated;

#[allow(
    dead_code,
    unused_imports,
    clippy::derivable_impls,
    clippy::derive_partial_eq_without_eq,
    clippy::extra_unused_lifetimes,
    clippy::size_of_in_element_count,
    clippy::needless_lifetimes,
    clippy::unnecessary_cast
)]
#[path = "./triggers_generated.rs"]
pub(crate) mod triggers_generated;

const MAGIC_NUMBER: &str = "#!AER-DAT4.0\r\n";

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Utf8(#[from] std::str::Utf8Error),

    #[error(transparent)]
    Roxmltree(#[from] roxmltree::Error),

    #[error(transparent)]
    ParseInt(#[from] std::num::ParseIntError),

    #[error("unsupported stream type {0}")]
    UnsupportedStreamType(String),

    #[error("bad magic number (expected \"#!AER-DAT4.0\\r\\n\", got \"{0}\")")]
    MagicNumber(String),

    #[error("empty description")]
    DescriptionEmpty,

    #[error("the description has no root node")]
    DescriptionRootNode,

    #[error("bad root node tag (expected \"dv\", got \"{0}\")")]
    DescriptionRootNodeTag(String),

    #[error("the description has no \"outInfo\" node")]
    DescriptionOutInfoNode,

    #[error("unexpected child in \"outInfo\" (expected \"node\", got {0})")]
    DescriptionStreamNodeTag(String),

    #[error("missing steam node ID")]
    DescriptionMissingStreamId,

    #[error("missing type for stream ID {0}")]
    DescriptionMissingType(u32),

    #[error("empty type for stream ID {0}")]
    DescriptionEmptyType(u32),

    #[error("missing sizeX attribute for stream ID {0}")]
    DescriptionMissingSizeX(u32),

    #[error("empty sizeX attribute for stream ID {0}")]
    DescriptionEmptySizeX(u32),

    #[error("missing sizeX attribute for stream ID {0}")]
    DescriptionMissingSizeY(u32),

    #[error("empty sizeX attribute for stream ID {0}")]
    DescriptionEmptySizeY(u32),

    #[error("missing info node for stream ID {0}")]
    DescriptionMissingInfoNode(u32),

    #[error("duplicated stream ID {0}")]
    DescriptionDuplicatedStreamId(u32),

    #[error("no stream found in the description")]
    DescriptionNoStream,
}

#[derive(Clone, Copy)]
pub enum StreamContent {
    Events,
    Frame,
    Imus,
    Triggers,
}

impl StreamContent {
    fn from(identifier: &str) -> Result<Self, Error> {
        match identifier {
            "EVTS" => Ok(StreamContent::Events),
            "FRME" => Ok(StreamContent::Frame),
            "IMUS" => Ok(StreamContent::Imus),
            "TRIG" => Ok(StreamContent::Triggers),
            _ => Err(Error::UnsupportedStreamType(identifier.to_owned())),
        }
    }
}

impl std::fmt::Display for StreamContent {
    fn fmt(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            formatter,
            "{}",
            match self {
                StreamContent::Events => "EVTS",
                StreamContent::Frame => "FRME",
                StreamContent::Imus => "IMUS",
                StreamContent::Triggers => "TRIG",
            }
        )
    }
}

#[derive(Clone, Copy)]
pub struct Stream {
    pub content: StreamContent,
    pub width: u16,
    pub height: u16,
}

pub struct Decoder {
    pub id_to_stream: std::collections::HashMap<u32, Stream>,
    file: std::fs::File,
    position: i64,
    compression: ioheader_generated::Compression,
    file_data_position: i64,
    raw_buffer: Vec<u8>,
    buffer: Vec<u8>,
}

impl Decoder {
    pub fn new<P: AsRef<std::path::Path>>(path: P) -> Result<Self, Error> {
        let mut decoder = Decoder {
            id_to_stream: std::collections::HashMap::new(),
            file: std::fs::File::open(path)?,
            position: 0i64,
            file_data_position: 0,
            compression: ioheader_generated::Compression::None,
            raw_buffer: Vec::new(),
            buffer: Vec::new(),
        };
        {
            let mut magic_number_buffer = [0; MAGIC_NUMBER.len()];
            decoder.file.read_exact(&mut magic_number_buffer)?;
            let magic_number = String::from_utf8_lossy(&magic_number_buffer).to_string();
            if magic_number != MAGIC_NUMBER {
                return Err(Error::MagicNumber(magic_number));
            }
            decoder.position += MAGIC_NUMBER.len() as i64;
        }
        let length = {
            let mut bytes = [0; 4];
            decoder.file.read_exact(&mut bytes)?;
            u32::from_le_bytes(bytes)
        };
        decoder.position += 4i64 + length as i64;
        {
            let mut buffer = std::vec![0; length as usize];
            decoder.file.read_exact(&mut buffer)?;
            let ioheader = unsafe { ioheader_generated::root_as_ioheader_unchecked(&buffer) };
            decoder.compression = ioheader.compression();
            decoder.file_data_position = ioheader.file_data_position();
            let description = match ioheader.description() {
                Some(content) => content,
                None => return Err(Error::DescriptionEmpty),
            };
            let document = roxmltree::Document::parse(description)?;
            let dv_node = match document.root().first_child() {
                Some(content) => content,
                None => return Err(Error::DescriptionRootNode),
            };
            if !dv_node.has_tag_name("dv") {
                return Err(Error::DescriptionRootNodeTag(
                    dv_node.tag_name().name().to_owned(),
                ));
            }
            let output_node = match dv_node.children().find(|node| {
                node.is_element()
                    && node.has_tag_name("node")
                    && node.attribute("name") == Some("outInfo")
            }) {
                Some(content) => content,
                None => return Err(Error::DescriptionOutInfoNode),
            };
            for stream_node in output_node.children() {
                if stream_node.is_element() && stream_node.has_tag_name("node") {
                    if !stream_node.has_tag_name("node") {
                        return Err(Error::DescriptionStreamNodeTag(
                            stream_node.tag_name().name().to_owned(),
                        ));
                    }
                    let stream_id = match stream_node.attribute("name") {
                        Some(content) => content,
                        None => return Err(Error::DescriptionMissingStreamId),
                    }
                    .parse::<u32>()?;
                    let identifier = match stream_node.children().find(|node| {
                        node.is_element()
                            && node.has_tag_name("attr")
                            && node.attribute("key") == Some("typeIdentifier")
                    }) {
                        Some(content) => match content.text() {
                            Some(content) => content,
                            None => return Err(Error::DescriptionEmptyType(stream_id)),
                        },
                        None => return Err(Error::DescriptionMissingType(stream_id)),
                    }
                    .to_string();
                    let mut width = 0u16;
                    let mut height = 0u16;
                    if identifier == "EVTS" || identifier == "FRME" {
                        let info_node = match stream_node.children().find(|node| {
                            node.is_element()
                                && node.has_tag_name("node")
                                && node.attribute("name") == Some("info")
                        }) {
                            Some(content) => content,
                            None => return Err(Error::DescriptionMissingInfoNode(stream_id)),
                        };
                        width = match info_node.children().find(|node| {
                            node.is_element()
                                && node.has_tag_name("attr")
                                && node.attribute("key") == Some("sizeX")
                        }) {
                            Some(content) => match content.text() {
                                Some(content) => content,
                                None => return Err(Error::DescriptionEmptySizeX(stream_id)),
                            },
                            None => return Err(Error::DescriptionMissingSizeX(stream_id)),
                        }
                        .parse::<u16>()?;
                        height = match info_node.children().find(|node| {
                            node.is_element()
                                && node.has_tag_name("attr")
                                && node.attribute("key") == Some("sizeY")
                        }) {
                            Some(content) => match content.text() {
                                Some(content) => content,
                                None => return Err(Error::DescriptionEmptySizeY(stream_id)),
                            },
                            None => return Err(Error::DescriptionMissingSizeY(stream_id)),
                        }
                        .parse::<u16>()?;
                    }
                    if decoder
                        .id_to_stream
                        .insert(
                            stream_id,
                            Stream {
                                content: StreamContent::from(&identifier)?,
                                width,
                                height,
                            },
                        )
                        .is_some()
                    {
                        return Err(Error::DescriptionDuplicatedStreamId(stream_id));
                    }
                }
            }
        }
        if decoder.id_to_stream.is_empty() {
            return Err(Error::DescriptionNoStream);
        }
        Ok(decoder)
    }
}

pub struct Packet<'a> {
    pub buffer: &'a std::vec::Vec<u8>,
    pub stream_id: u32,
    pub stream: Stream,
}

#[derive(thiserror::Error, Debug)]
pub enum PacketError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    Flatbuffers(#[from] flatbuffers::InvalidFlatbuffer),

    #[error("unknown compression algorithm")]
    CompressionAlgorithm,

    #[error("unknown packet stream ID {0}")]
    UnknownPacketStreamId(u32),

    #[error("bad packet prefix for stream ID {id} (expected \"{expected}\", got \"{got}\")")]
    BadPacketPrefix {
        id: u32,
        expected: String,
        got: String,
    },

    #[error("empty events packet")]
    EmptyEventsPacket,

    #[error("missing packet size prefix")]
    MissingPacketSizePrefix,

    #[error("unknown frame format")]
    UnknownFrameFormat,

    #[error("unknown trigger source")]
    UnknownTriggerSource,

    #[error("x overflow (x={x} should be larger than 0 and strictly smaller than width={width})")]
    XOverflow { x: i16, width: u16 },

    #[error(
        "y overflow (y={y} should be larger than 0 and strictly smaller than height={height})"
    )]
    YOverflow { y: i16, height: u16 },
}

impl Decoder {
    pub fn next(&mut self) -> Result<Option<Packet>, PacketError> {
        if self.file_data_position > -1 && self.position == self.file_data_position {
            return Ok(None);
        }
        let (stream_id, length) = {
            let mut bytes = [0; 8];
            self.file.read_exact(&mut bytes)?;
            let stream_id = u32::from_le_bytes(bytes[0..4].try_into().expect("four bytes"));
            let length = u32::from_le_bytes(bytes[4..8].try_into().expect("four bytes"));
            (stream_id, length)
        };
        self.position += 8i64 + length as i64;
        self.raw_buffer.resize(length as usize, 0u8);
        self.file.read_exact(&mut self.raw_buffer)?;
        match self.compression {
            ioheader_generated::Compression::None => {
                std::mem::swap(&mut self.raw_buffer, &mut self.buffer)
            }
            ioheader_generated::Compression::Lz4 | ioheader_generated::Compression::Lz4High => {
                let mut decoder = lz4::Decoder::new(&self.raw_buffer[..])?;
                self.buffer.clear();
                decoder.read_to_end(&mut self.buffer)?;
            }
            ioheader_generated::Compression::Zstd | ioheader_generated::Compression::ZstdHigh => {
                let mut decoder = lz4::Decoder::new(&self.raw_buffer[..])?;
                self.buffer.clear();
                decoder.read_to_end(&mut self.buffer)?;
            }
            _ => return Err(PacketError::CompressionAlgorithm),
        }
        let stream = self
            .id_to_stream
            .get(&stream_id)
            .ok_or(PacketError::UnknownPacketStreamId(stream_id))?;
        let expected_content_string = stream.content.to_string();
        if !flatbuffers::buffer_has_identifier(&self.buffer, &expected_content_string, true) {
            let expected_length = expected_content_string.len();
            let offset = flatbuffers::SIZE_SIZEPREFIX + flatbuffers::SIZE_UOFFSET;
            return Err(PacketError::BadPacketPrefix {
                id: stream_id,
                expected: expected_content_string,
                got: if self.buffer.len() >= offset {
                    String::from_utf8_lossy(
                        &self.buffer
                            [offset..offset + expected_length.min(self.buffer.len() - offset)],
                    )
                    .into_owned()
                } else {
                    "".to_owned()
                }
                .to_string(),
            });
        }
        Ok(Some(Packet {
            buffer: &self.buffer,
            stream_id,
            stream: *stream,
        }))
    }
}
