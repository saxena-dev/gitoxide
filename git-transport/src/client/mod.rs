#[cfg(feature = "blocking-client")]
mod blocking_io;
#[cfg(all(feature = "blocking-client", feature = "http-client-curl"))]
pub use blocking_io::http;
#[cfg(feature = "blocking-client")]
pub use blocking_io::{
    connect, file, git,
    request::{ExtendedBufRead, HandleProgress, RequestWriter},
    ssh, SetServiceResponse, Transport, TransportV2Ext,
};
#[cfg(feature = "blocking-client")]
#[doc(inline)]
pub use connect::connect;

#[cfg(all(not(feature = "blocking-client"), feature = "async-client"))]
mod async_io;
#[cfg(all(not(feature = "blocking-client"), feature = "async-client"))]
pub use async_io::SetServiceResponse;

mod non_io_types;
pub use non_io_types::{Error, Identity, MessageKind, WriteMode};

///
pub mod capabilities;
#[doc(inline)]
pub use capabilities::Capabilities;
