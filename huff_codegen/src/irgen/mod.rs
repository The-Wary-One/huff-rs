/// Constant Bytecode Generation Module
pub mod constants;

/// Statement Bytecode Generation Module
pub mod statements;

/// Argument Call Module
pub mod arg_calls;

/// Builtin Function Call Producers
pub mod builtins;

/// Prelude wraps common utilities.
pub mod prelude {
    pub use super::{builtins::*, arg_calls::*, constants::*, statements::*};
}
