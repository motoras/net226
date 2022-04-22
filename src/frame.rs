use std::fmt::{Debug, Display, Formatter, Result};
use std::mem::size_of;

pub const JOIN: u16 = 0;
pub const HEARTBEAT: u16 = 1;
pub const LEAVE: u16 = 2;

#[derive(Debug)]
pub enum Frame {
    Join { uuid: u128, port: u16, flags: u8 },
    Heartbeat { uuid: u128, port: u16, flags: u8 },
    Leave { uuid: u128, flags: u8 },
}

impl Display for Frame {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        match &self {
            Frame::Join { uuid, port, flags } => {
                write!(
                    f,
                    "Hello from {:x}:{}, Extra {}",
                    uuid,
                    port,
                    flags % 2 == 0
                )
            }
            Frame::Heartbeat { uuid, port, flags } => {
                write!(
                    f,
                    "Heartbeat from {:x}:{}, Extra {}",
                    uuid,
                    port,
                    flags % 2 == 0
                )
            }
            Frame::Leave { uuid, flags } => {
                write!(f, "Bye from {:x}, Extra {}", uuid, flags % 2 == 0)
            }
        }
    }
}

pub struct FrameIterator<'a> {
    pos: usize,
    bytes: &'a [u8],
}

impl<'a> FrameIterator<'a> {
    pub fn new(bytes: &'a [u8]) -> FrameIterator<'a> {
        FrameIterator { bytes, pos: 0usize }
    }
}

impl<'a> Iterator for FrameIterator<'a> {
    type Item = Frame;

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos + 19 <= self.bytes.len() {
            let msg_id = u16::from_be_bytes(
                self.bytes[self.pos..self.pos + size_of::<u16>()]
                    .try_into()
                    .unwrap(),
            );
            self.pos += size_of::<u16>();
            match msg_id {
                JOIN => {
                    let uuid = u128::from_be_bytes(
                        self.bytes[self.pos..self.pos + size_of::<u128>()]
                            .try_into()
                            .unwrap(),
                    );
                    self.pos += size_of::<u128>();
                    let port = u16::from_be_bytes(
                        self.bytes[self.pos..self.pos + size_of::<u16>()]
                            .try_into()
                            .unwrap(),
                    );
                    self.pos += size_of::<u16>();
                    let flags = u8::from_be_bytes(
                        self.bytes[self.pos..self.pos + size_of::<u8>()]
                            .try_into()
                            .unwrap(),
                    );
                    self.pos += size_of::<u8>();
                    self.pos += size_of::<u8>();
                    Some(Frame::Join { uuid, port, flags })
                }
                HEARTBEAT => {
                    let uuid = u128::from_be_bytes(
                        self.bytes[self.pos..self.pos + size_of::<u128>()]
                            .try_into()
                            .unwrap(),
                    );
                    self.pos += size_of::<u128>();
                    let port = u16::from_be_bytes(
                        self.bytes[self.pos..self.pos + size_of::<u16>()]
                            .try_into()
                            .unwrap(),
                    );
                    self.pos += size_of::<u16>();
                    let flags = u8::from_be_bytes(
                        self.bytes[self.pos..self.pos + size_of::<u8>()]
                            .try_into()
                            .unwrap(),
                    );
                    self.pos += size_of::<u8>();
                    self.pos += size_of::<u8>();
                    Some(Frame::Heartbeat { uuid, port, flags })
                }
                LEAVE => {
                    let uuid = u128::from_be_bytes(
                        self.bytes[self.pos..self.pos + size_of::<u128>()]
                            .try_into()
                            .unwrap(),
                    );
                    self.pos += size_of::<u128>();
                    let flags = u8::from_be_bytes(
                        self.bytes[self.pos..self.pos + size_of::<u8>()]
                            .try_into()
                            .unwrap(),
                    );
                    self.pos += size_of::<u8>();
                    self.pos += size_of::<u8>();
                    Some(Frame::Leave { uuid, flags })
                }

                _ => None,
            }
        } else {
            None
        }
    }
}
