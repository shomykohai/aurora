use aurora_protocol::board::Board;

pub mod boards;

pub use boards::*;

pub const ALL: &[&Board] = &[&boards::xinmeng::XINMENG_M75];

pub fn by_id(id: &str) -> Option<&'static Board> {
    ALL.iter().copied().find(|board| board.id.eq_ignore_ascii_case(id))
}
