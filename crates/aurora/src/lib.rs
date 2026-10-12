mod device;

pub use aurora_boards as boards;
pub use aurora_protocol::{
    Board,
    Cap,
    CapId,
    Error,
    KeyDesc,
    Quirks,
    Region,
    RegionRequest,
    Result,
    TransportKind,
    UsbId,
    board,
    cap,
    common,
    keys,
    leds,
    op,
};
pub use device::Device;
