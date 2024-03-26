mod rx_stream;
mod tx_stream;

pub use rx_stream::{RxStream, RxStreamBuilder, RxStreamReader};
pub use tx_stream::{TxStream, TxStreamBuilder, TxStreamWriter};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OtwFormat {
    ComplexInt16,
    ComplexInt12,
    ComplexInt8,
    Int16,
    Int8,
}

impl OtwFormat {
    pub fn as_str(&self) -> &'static str {
        match self {
            OtwFormat::ComplexInt16 => "sc16",
            OtwFormat::ComplexInt12 => "sc12",
            OtwFormat::ComplexInt8 => "sc8",
            OtwFormat::Int16 => "s16",
            OtwFormat::Int8 => "s8",
        }
    }
}
