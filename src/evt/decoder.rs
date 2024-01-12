use std::io::Read;
use std::io::Seek;

use crate::utilities;

pub enum Version {
    Evt2,
    Evt21,
    Evt3,
}

impl Version {
    pub fn from_string(string: &str) -> Result<Version, Error> {
        match string {
            "EVT2" => Ok(Version::Evt2),
            "EVT2.1" => Ok(Version::Evt21),
            "EVT3" => Ok(Version::Evt3),
            string => Err(Error::UnknownVersion(string.to_owned())),
        }
    }
}

enum State {
    Evt2 {
        t: u64,
        t_high: u64,
        t_offset: u64,
        t_without_offset: u64,
    },
    Evt21 {},
    Evt3 {
        t: u64,
        overflows: u32,
        previous_msb_t: u16,
        previous_lsb_t: u16,
        x: u16,
        y: u16,
    },
}

pub struct Decoder {
    pub width: u16,
    pub height: u16,
    file: std::fs::File,
    raw_buffer: Vec<u8>,
    event_buffer: Vec<neuromorphic_types::DvsEvent<u64, u16, u16>>,
    trigger_buffer: Vec<neuromorphic_types::TriggerEvent<u64, u8>>,
    state: State,
    polarity: neuromorphic_types::DvsPolarity,
}

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(
        "the header has no size information (width and height) and no size fallback was provided"
    )]
    MissingSize,

    #[error("the header has no version information (EVT2, EVT2.1, or EVT3) and no versions fallback was provided")]
    MissingVersion,

    #[error("unknown version \"{0}\" (supports \"EVT2\", \"EVT2.1\", or \"EVT3\")")]
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
        let mut file = std::fs::File::open(path)?;
        file.seek(std::io::SeekFrom::Start(header.length))?;
        let version = match header.version {
            Some(version) => match version.as_str() {
                "2" => Version::Evt2,
                "2.1" => Version::Evt21,
                "3" => Version::Evt3,
                _ => return Err(Error::UnknownVersion(version)),
            },
            None => match version_fallback {
                Some(version) => version,
                None => return Err(Error::MissingVersion),
            },
        };
        Ok(Decoder {
            width: size.0,
            height: size.1,
            file,
            raw_buffer: vec![0u8; utilities::BUFFER_SIZE],
            event_buffer: Vec::new(),
            trigger_buffer: Vec::new(),
            state: match version {
                Version::Evt2 => State::Evt2 {
                    t: 0,
                    t_high: 0,
                    t_offset: 0,
                    t_without_offset: 0,
                },
                Version::Evt21 => State::Evt21 {},
                Version::Evt3 => State::Evt3 {
                    t: 0,
                    overflows: 0,
                    previous_msb_t: 0,
                    previous_lsb_t: 0,
                    x: 0,
                    y: 0,
                },
            },
            polarity: neuromorphic_types::DvsPolarity::Off,
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
    pub fn next(
        &mut self,
    ) -> Result<
        Option<(
            &Vec<neuromorphic_types::DvsEvent<u64, u16, u16>>,
            &Vec<neuromorphic_types::TriggerEvent<u64, u8>>,
        )>,
        PacketError,
    > {
        let read = self.file.read(&mut self.raw_buffer)?;
        if read == 0 {
            return Ok(None);
        }
        self.event_buffer.clear();
        self.trigger_buffer.clear();
        match self.state {
            State::Evt2 {
                ref mut t,
                ref mut t_high,
                ref mut t_offset,
                ref mut t_without_offset,
            } => {
                for index in 0..read / 4 {
                    let word = u32::from_le_bytes([
                        self.raw_buffer[index * 4],
                        self.raw_buffer[index * 4 + 1],
                        self.raw_buffer[index * 4 + 2],
                        self.raw_buffer[index * 4 + 3],
                    ]);

                    match word >> 28 {
                        0b0000 | 0b0001 => {
                            *t = (*t_high
                                + ((word & 0b1111110000000000000000000000_u32) >> 22) as u64)
                                .max(*t);
                            let x = ((word & 0b1111111111100000000000_u32) >> 11)  as u16;
                            if x >= self.width {
                                return Err(PacketError::XOverflow {
                                    x,
                                    width: self.width,
                                });
                            }
                            let y = (word & 0b0000000000011111111111_u32) as u16;
                            if y >= self.height {
                                return Err(PacketError::YOverflow {
                                    y,
                                    height: self.height,
                                });
                            }
                            self.event_buffer.push(neuromorphic_types::DvsEvent {
                                t: *t,
                                x,
                                y,
                                polarity: if (word >> 28) & 0b1 > 0 {
                                    neuromorphic_types::DvsPolarity::On
                                } else {
                                    neuromorphic_types::DvsPolarity::Off
                                },
                            });
                        }
                        0b1000 => {
                            let candidate_t_without_offset =
                                ((word & 0b1111111111111111111111111111_u32) as u64) << 6;
                            if candidate_t_without_offset < *t_without_offset {
                                *t_offset += 1u64 << 34;
                            }
                            *t_without_offset = candidate_t_without_offset;
                            *t_high = *t_without_offset + *t_offset;
                        }
                        0b1010 => {
                            *t = (*t_high
                                + ((word & 0b1111110000000000000000000000_u32) >> 22) as u64)
                                .max(*t);
                            self.trigger_buffer.push(neuromorphic_types::TriggerEvent {
                                t: *t,
                                id: ((word & 0b1111100000000) >> 8) as u8,
                                polarity: if (word & 1) > 0 {
                                    neuromorphic_types::TriggerPolarity::Rising
                                } else {
                                    neuromorphic_types::TriggerPolarity::Falling
                                },
                            })
                        }
                        #[allow(clippy::manual_range_patterns)]
                        0b1110 | 0b1111 => (),
                        _ => (),
                    }
                }
            }
            State::Evt21 {} => {}
            State::Evt3 {
                ref mut t,
                ref mut overflows,
                ref mut previous_msb_t,
                ref mut previous_lsb_t,
                ref mut x,
                ref mut y,
            } => {
                for index in 0..read / 2 {
                    let word = u16::from_le_bytes([
                        self.raw_buffer[index * 2],
                        self.raw_buffer[index * 2 + 1],
                    ]);
                    match word >> 12 {
                        0b0000 => {
                            let candidate_y = word & 0b11111111111;
                            if candidate_y < self.height {
                                *y = candidate_y;
                            } else {
                                return Err(PacketError::YOverflow {
                                    y: candidate_y,
                                    height: self.height,
                                });
                            }
                        }
                        0b0001 => (),
                        0b0010 => {
                            let candidate_x = word & 0b11111111111;
                            if candidate_x < self.width {
                                *x = candidate_x;
                            } else {
                                return Err(PacketError::XOverflow {
                                    x: candidate_x,
                                    width: self.width,
                                });
                            }
                            self.polarity = if (word & (1 << 11)) > 0 {
                                neuromorphic_types::DvsPolarity::On
                            } else {
                                neuromorphic_types::DvsPolarity::Off
                            };
                            self.event_buffer.push(neuromorphic_types::DvsEvent {
                                t: *t,
                                x: *x,
                                y: *y,
                                polarity: self.polarity,
                            });
                        }
                        0b0011 => {
                            let candidate_x = word & 0b11111111111;
                            if candidate_x < self.width {
                                *x = candidate_x;
                            } else {
                                return Err(PacketError::XOverflow {
                                    x: candidate_x,
                                    width: self.width,
                                });
                            }
                            self.polarity = if (word & (1 << 11)) > 0 {
                                neuromorphic_types::DvsPolarity::On
                            } else {
                                neuromorphic_types::DvsPolarity::Off
                            };
                        }
                        0b0100 => {
                            let set = word & ((1 << std::cmp::min(12, self.width - *x)) - 1);
                            for bit in 0..12 {
                                if (set & (1 << bit)) > 0 {
                                    self.event_buffer.push(neuromorphic_types::DvsEvent {
                                        t: *t,
                                        x: *x + bit,
                                        y: *y,
                                        polarity: self.polarity,
                                    });
                                }
                            }
                            *x = (*x + 12).min(self.width - 1);
                        }
                        0b0101 => {
                            let set = word & ((1 << std::cmp::min(8, self.width - *x)) - 1);
                            for bit in 0..8 {
                                if (set & (1 << bit)) > 0 {
                                    self.event_buffer.push(neuromorphic_types::DvsEvent {
                                        t: *t,
                                        x: *x + bit,
                                        y: *y,
                                        polarity: self.polarity,
                                    });
                                }
                            }
                            *x = (*x + 8).min(self.width - 1);
                        }
                        0b0110 => {
                            let lsb_t = word & 0b111111111111;
                            if lsb_t != *previous_lsb_t {
                                *previous_lsb_t = lsb_t;
                                let candidate_t = (((*previous_lsb_t as u32)
                                    | ((*previous_msb_t as u32) << 12))
                                    as u64)
                                    | ((*overflows as u64) << 24);
                                if candidate_t >= *t {
                                    *t = candidate_t;
                                }
                            }
                        }
                        0b0111 => (),
                        0b1000 => {
                            let msb_t = word & 0b111111111111;
                            if msb_t != *previous_msb_t {
                                if msb_t > *previous_msb_t {
                                    if (msb_t - *previous_msb_t) < ((1 << 12) - 2) {
                                        *previous_lsb_t = 0;
                                        *previous_msb_t = msb_t;
                                    }
                                } else if (*previous_msb_t - msb_t) > ((1 << 12) - 2) {
                                    *overflows += 1;
                                    *previous_lsb_t = 0;
                                    *previous_msb_t = msb_t;
                                }
                                let candidate_t = (((*previous_lsb_t as u32)
                                    | ((*previous_msb_t as u32) << 12))
                                    as u64)
                                    | ((*overflows as u64) << 24);
                                if candidate_t >= *t {
                                    *t = candidate_t;
                                }
                            }
                        }
                        0b1001 => (),
                        0b1010 => self.trigger_buffer.push(neuromorphic_types::TriggerEvent {
                            t: *t,
                            id: ((word >> 8) & 0b1111) as u8,
                            polarity: if (word & 1) > 0 {
                                neuromorphic_types::TriggerPolarity::Rising
                            } else {
                                neuromorphic_types::TriggerPolarity::Falling
                            },
                        }),
                        #[allow(clippy::manual_range_patterns)]
                        0b1011 | 0b1100 | 0b1101 | 0b1110 | 0b1111 => (),
                        _ => (),
                    }
                }
            }
        }
        Ok(Some((&self.event_buffer, &self.trigger_buffer)))
    }
}
