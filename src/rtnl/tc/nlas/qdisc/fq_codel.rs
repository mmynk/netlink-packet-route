// SPDX-License-Identifier: MIT

use netlink_packet_utils::{DecodeError, nla::Nla};

use crate::{nlas::tc::{ATTR_LEN, NLA_HEADER_LEN}, TCA_FQ_CODEL};

pub const FQ_CODEL: &str = "fq_codel";
pub const FQ_CODEL_LEN: usize = 64;

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct FqCodel {
    pub target: u32,
    pub limit: u32,
    pub interval: u32,
    pub ecn: u32,
    pub flows: u32,
    pub quantum: u32,
    pub ce_threshold: u32,
    pub drop_batch_size: u32,
    pub memory_limit: u32,

    // The order of the fields is not as per the enum `TcaFqCodel`
    // Thus, we need to track the order in order to reproduce the buffer in `emit`.
    pub order: Vec<TcaFqCodel>,
}

impl FqCodel {
    pub fn new(data: &[u8]) -> Result<Self, DecodeError> {
        unmarshal_fq_codel(data)
    }
}

impl Nla for FqCodel {
    fn value_len(&self) -> usize {
        FQ_CODEL_LEN
    }

    fn kind(&self) -> u16 {
        TCA_FQ_CODEL
    }

    fn emit_value(&self, buffer: &mut [u8]) {
        let mut offset = 0;
        let values = [
            self.target,
            self.limit,
            self.interval,
            self.ecn,
            self.flows,
            self.quantum,
            self.ce_threshold,
            self.drop_batch_size,
            self.memory_limit,
        ];
        let length = 8u16;
        for field in &self.order {
            // length
            buffer[offset..offset + 2].copy_from_slice(&length.to_ne_bytes());
            // kind
            let kind = field.clone() as u16;
            buffer[offset + 2..offset + 4].copy_from_slice(&kind.to_ne_bytes());
            offset += 4;
            // value
            let value = match *field {
                TcaFqCodel::Target => values[0],
                TcaFqCodel::Limit => values[1],
                TcaFqCodel::Interval => values[2],
                TcaFqCodel::Ecn => values[3],
                TcaFqCodel::Flows => values[4],
                TcaFqCodel::Quantum => values[5],
                TcaFqCodel::CeThreshold => values[6],
                TcaFqCodel::DropBatchSize => values[7],
                TcaFqCodel::MemoryLimit => values[8],
                _ => unreachable!(),
            };
            buffer[offset..offset + ATTR_LEN].copy_from_slice(&value.to_ne_bytes());
            offset += ATTR_LEN;
        }
    }
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum TcaFqCodel {
    #[default]
    Unspec = 0,
    Target,
    Limit,
    Interval,
    Ecn,
    Flows,
    Quantum,
    CeThreshold,
    DropBatchSize,
    MemoryLimit,
    Max,
}

impl From<u16> for TcaFqCodel {
    fn from(v: u16) -> Self {
        match v {
            0 => TcaFqCodel::Unspec,
            1 => TcaFqCodel::Target,
            2 => TcaFqCodel::Limit,
            3 => TcaFqCodel::Interval,
            4 => TcaFqCodel::Ecn,
            5 => TcaFqCodel::Flows,
            6 => TcaFqCodel::Quantum,
            7 => TcaFqCodel::CeThreshold,
            8 => TcaFqCodel::DropBatchSize,
            9 => TcaFqCodel::MemoryLimit,
            _ => TcaFqCodel::Max,
        }
    }
}

fn unmarshal_fq_codel_attr(data: &[u8]) -> Result<(u16, u32), DecodeError> {
    if data.len() < NLA_HEADER_LEN {
        return Err(DecodeError::from("fq_codel: invalid data"));
    }

    let length = u16::from_ne_bytes([data[0], data[1]]) as usize;
    let kind = u16::from_ne_bytes([data[2], data[3]]);

    if length > data.len() {
        return Err(DecodeError::from("fq_codel: invalid data"));
    }

    if length == 0 {
        return Err(DecodeError::from("fq_codel: empty data"));
    }

    if length < NLA_HEADER_LEN {
        return Err(DecodeError::from("fq_codel: invalid data"));
    }

    let payload_length = length - NLA_HEADER_LEN;
    if payload_length != ATTR_LEN {
        return Err(DecodeError::from("fq_codel: invalid data"));
    }
    let mut bytes = [0u8; ATTR_LEN];
    bytes.copy_from_slice(&data[NLA_HEADER_LEN..NLA_HEADER_LEN + ATTR_LEN]);

    Ok((kind, u32::from_ne_bytes(bytes)))
}

pub fn unmarshal_fq_codel(data: &[u8]) -> Result<FqCodel, DecodeError> {
    let mut fq = FqCodel::default();

    let length = data.len();
    let mut offset = 0;
    while offset < length {
        let buf = &data[offset..];
        let (kind, attr) = unmarshal_fq_codel_attr(buf)?;
        let kind = TcaFqCodel::from(kind);
        fq.order.push(kind.clone());
        match kind {
            TcaFqCodel::Target => fq.target = attr,
            TcaFqCodel::Limit => fq.limit = attr,
            TcaFqCodel::Interval => fq.interval = attr,
            TcaFqCodel::Ecn => fq.ecn = attr,
            TcaFqCodel::Flows => fq.flows = attr,
            TcaFqCodel::Quantum => fq.quantum = attr,
            TcaFqCodel::CeThreshold => fq.ce_threshold = attr,
            TcaFqCodel::DropBatchSize => fq.drop_batch_size = attr,
            TcaFqCodel::MemoryLimit => fq.memory_limit = attr,
            _ => return Err(DecodeError::from("fq_codel: unknown attribute")),
        }
        offset += NLA_HEADER_LEN + ATTR_LEN;
    }
    Ok(fq)
}
