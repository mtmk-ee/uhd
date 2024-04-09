mod read;
mod write;

pub use read::ChannelConfiguration;
pub use write::ChannelConfigurationBuilder;

pub(crate) const TX_DIR: usize = 0;
pub(crate) const RX_DIR: usize = 1;