use std::io::Read;
use std::io::Seek;

use crate::utilities;

pub enum Version {
    Dat1,
    Dat2,
}

impl Version {
    pub fn from_string(string: &str) -> Result<Version, Error> {
        match string {
            "DAT1" => Ok(Version::Dat1),
            "DAT2" => Ok(Version::Dat2),
            string => Err(Error::UnknownVersion(string.to_owned())),
        }
    }
}

#[repr(C, packed)]
#[derive(Debug, Copy, Clone)]
pub struct Event {
    pub t: u64,
    pub x: u16,
    pub y: u16,
    pub payload: u8,
}

#[derive(Debug, Copy, Clone)]
pub enum Type {
    Event2d,
    EventCd,
    EventExtTrigger,
}

pub struct Decoder {
    pub width: u16,
    pub height: u16,
    pub event_type: Type,
    version: Version,
    file: std::fs::File,
    raw_buffer: Vec<u8>,
    event_buffer: Vec<Event>,
    t: u64,
    offset: u64,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("the header has no size information and no size fallback was provided")]
    MissingSize,

    #[error("event type ({0}) not supported")]
    UnsupportedType(u8),

    #[error("event size ({0}) not supported")]
    UnsupportedEventSize(u8),

    #[error("the header has no version information (DAT1 or DAT2) and no versions fallback was provided")]
    MissingVersion,

    #[error("unknown version \"{0}\" (supports \"DAT1\" or \"DAT2\")")]
    UnknownVersion(String),
}

impl Decoder {
    pub fn new<P: AsRef<std::path::Path>>(
        path: P,
        size_fallback: Option<(u16, u16)>,
        version_fallback: Option<Version>,
    ) -> Result<Self, Error> {
        let header = utilities::read_header(
            &mut std::io::BufReader::new(std::fs::File::open(&path)?),
            '%',
        )?;
        let size = match header.size {
            Some(size) => size,
            None => match size_fallback {
                Some(size) => size,
                None => return Err(Error::MissingSize),
            },
        };
        let version = match header.version {
            Some(version) => match version.as_str() {
                "1" => Version::Dat1,
                "2" => Version::Dat2,
                _ => return Err(Error::UnknownVersion(version)),
            },
            None => match version_fallback {
                Some(version) => version,
                None => return Err(Error::MissingVersion),
            },
        };
        let mut file = std::fs::File::open(path)?;
        file.seek(std::io::SeekFrom::Start(header.length))?;
        let event_type = {
            let mut type_and_size = [0u8; 2];
            file.read(&mut type_and_size)?;
            if type_and_size[1] != 8 {
                return Err(Error::UnsupportedEventSize(type_and_size[1]));
            }
            match type_and_size[0] {
                0x00 => Type::Event2d,
                0x0C => Type::EventCd,
                0x0E => Type::EventExtTrigger,
                event_type => return Err(Error::UnsupportedType(event_type)),
            }
        };
        Ok(Decoder {
            width: size.0,
            height: size.1,
            event_type,
            version,
            file,
            raw_buffer: vec![0u8; utilities::BUFFER_SIZE],
            event_buffer: Vec::new(),
            t: 0,
            offset: 0,
        })
    }
}

#[derive(thiserror::Error, Debug)]
pub enum PacketError {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("x overflow (x={x} should be strictly smaller than width={width})")]
    XOverflow { x: u16, width: u16 },

    #[error("y overflow (y={y} should be strictly smaller than height={height})")]
    YOverflow { y: u16, height: u16 },
}

impl Decoder {
    pub fn next(&mut self) -> Result<Option<&Vec<Event>>, PacketError> {
        let read = self.file.read(&mut self.raw_buffer)?;
        if read == 0 {
            return Ok(None);
        }
        self.event_buffer.clear();
        self.event_buffer.reserve(read / 8);

        match self.version {
            Version::Dat1 => {
                for index in 0..read / 8 {
                    let word = u64::from_le_bytes(
                        self.raw_buffer[index * 8..(index + 1) * 8]
                            .try_into()
                            .expect("8 bytes"),
                    );
                    let mut candidate_t = (word & 0xffffffff_u64) + self.offset;
                    if candidate_t < self.t {
                        if self.t - candidate_t > (1_u64 << 31) {
                            candidate_t += 1_u64 << 32;
                            self.offset += 1_u64 << 32;
                            self.t = candidate_t;
                        }
                    } else {
                        self.t = candidate_t;
                    }
                    let x = ((word >> 32) & 0b111111111_u64) as u16;
                    let y = ((word >> 41) & 0b11111111_u64) as u16;
                    if x >= self.width {
                        return Err(PacketError::XOverflow {
                            x,
                            width: self.width,
                        }
                        .into());
                    }
                    if y >= self.height {
                        return Err(PacketError::YOverflow {
                            y,
                            height: self.height,
                        }
                        .into());
                    }
                    self.event_buffer.push(Event {
                        t: self.t,
                        x,
                        y,
                        payload: ((word >> 49) & 0b1111) as u8,
                    });
                }
            }
            Version::Dat2 => {
                for index in 0..read / 8 {
                    let word = u64::from_le_bytes(
                        self.raw_buffer[index * 8..(index + 1) * 8]
                            .try_into()
                            .expect("8 bytes"),
                    );
                    let mut candidate_t = (word & 0xffffffff_u64) + self.offset;
                    if candidate_t < self.t {
                        if self.t - candidate_t > (1_u64 << 31) {
                            candidate_t += 1_u64 << 32;
                            self.offset += 1_u64 << 32;
                            self.t = candidate_t;
                        }
                    } else {
                        self.t = candidate_t;
                    }
                    let x = ((word >> 32) & 0b11111111111111_u64) as u16;
                    let y = ((word >> 46) & 0b11111111111111_u64) as u16;
                    if x >= self.width {
                        return Err(PacketError::XOverflow {
                            x,
                            width: self.width,
                        }
                        .into());
                    }
                    if y >= self.height {
                        return Err(PacketError::YOverflow {
                            y,
                            height: self.height,
                        }
                        .into());
                    }
                    self.event_buffer.push(Event {
                        t: self.t,
                        x,
                        y,
                        payload: (word >> 60) as u8,
                    });
                }
            }
        }
        Ok(Some(&self.event_buffer))
    }
}
