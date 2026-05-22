//! System-level adapters: wall clock, OS-backed RNG, in-memory pair-code store.

mod clock;
mod pair_codes;
mod random;

pub use clock::SystemClock;
pub use pair_codes::MemoryPairCodeStore;
pub use random::OsRandom;
