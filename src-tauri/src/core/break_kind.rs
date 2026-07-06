use std::fmt::{self, Display};

/// Break category shared by scheduler commands and break-specific features.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BreakKind {
    Mini,
    Long,
}

impl Display for BreakKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BreakKind::Mini => write!(f, "Mini"),
            BreakKind::Long => write!(f, "Long"),
        }
    }
}
