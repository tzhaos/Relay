use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

macro_rules! domain_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
        pub struct $name(Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(Uuid::new_v4())
            }

            pub fn from_uuid(id: Uuid) -> Self {
                Self(id)
            }

            pub fn as_uuid(self) -> Uuid {
                self.0
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self::new()
            }
        }

        impl fmt::Display for $name {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

domain_id!(TaskId);
domain_id!(ProjectId);
domain_id!(WorktreeId);
domain_id!(TerminalSessionId);
domain_id!(AgentSessionId);
domain_id!(ReviewCommentId);
domain_id!(PreviewTargetId);
