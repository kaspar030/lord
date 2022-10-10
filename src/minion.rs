use std::time;

// pub struct MinionId(pub usize);
// pub struct MinionKey(pub String);

#[derive(Debug)]
pub enum MinionState {
    Pending,
    // Adopted,
    // Denied,
}

#[derive(Debug)]
pub struct Minion {
    // id: MinionId,
    // key: MinionKey,
    pub state: MinionState,
    pub last_seen: time::Instant,
}

impl Minion {
    pub fn new() -> Self {
        Self {
            state: MinionState::Pending,
            last_seen: time::Instant::now(),
        }
    }
}
