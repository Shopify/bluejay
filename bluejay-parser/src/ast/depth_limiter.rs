use crate::ast::ParseError;

pub const DEFAULT_MAX_DEPTH: usize = 2000;

/// A depth limiter is used to limit the depth of the AST. This is useful to prevent stack overflows.
/// This intentionally does not implement `Clone` or `Copy` to passing this down the call stack without bumping.
pub struct DepthLimiter {
    max_depth: usize,
    current_depth: usize,
}

impl Default for DepthLimiter {
    fn default() -> Self {
        Self {
            max_depth: DEFAULT_MAX_DEPTH,
            current_depth: 0,
        }
    }
}

impl DepthLimiter {
    pub fn new(max_depth: usize) -> Self {
        Self {
            max_depth,
            current_depth: 0,
        }
    }

    pub fn bump(&self) -> Result<Self, ParseError> {
        if self.current_depth >= self.max_depth {
            Err(ParseError::MaxDepthExceeded)
        } else {
            Ok(Self {
                max_depth: self.max_depth,
                current_depth: self.current_depth + 1,
            })
        }
    }
}
