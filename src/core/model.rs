use std::{
    fmt::Display,
    time::{SystemTime, UNIX_EPOCH},
};

#[macro_export]
macro_rules! create_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy)]
        #[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
        #[cfg_attr(feature = "sqlx", sqlx(transparent))]
        pub struct $name(sqlx::types::Uuid);

        impl $name {
            pub fn new() -> Self {
                Self(uuid::Uuid::now_v7())
            }

            pub fn timestamp(&self) -> uuid::Timestamp {
                // should be enforced at build time
                assert!(self.0.get_version_num() == 7);
                self.0.get_timestamp().unwrap()
            }
        }

        impl std::convert::TryFrom<uuid::Uuid> for $name {
            type Error = ModelError;

            fn try_from(value: uuid::Uuid) -> Result<Self, Self::Error> {
                if value.get_version_num() != 7 {
                    Err(ModelError::InvalidId)
                } else {
                    Ok(Self(value))
                }
            }
        }
    };
}

pub enum ModelError {
    InvalidId,
}

create_id!(UserId);
create_id!(ProjectId);
create_id!(EntryId);

#[derive(Debug)]
pub struct Version(u16);
/// hardcoded in lib
#[derive(Debug)]
pub struct Revision; // make static string
#[derive(Debug)]
pub struct Date(u64);

impl Date {
    fn now() -> Self {
        Self(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as u64,
        )
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct Username(String);
impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type, sqlx::FromRow, sqlx::Encode))]
pub struct Author {
    id: UserId,
    name: Username,
}
impl Author {
    pub fn create(name: Username) -> Self {
        Self {
            id: UserId::new(),
            name,
        }
    }
    pub fn id(&self) -> UserId {
        self.id
    }
    pub fn name(&self) -> &Username {
        &self.name
    }
}
#[derive(Debug)]
pub struct Project {
    id: ProjectId,
    mata: Metadata,
}

#[derive(Debug)]
pub struct Metadata {
    revision: Revision,
    version: Version,
    author: UserId,
    created: Date,
}
#[derive(Debug)]
pub struct Log {
    id: EntryId,
    meta: Metadata,
    text: String,
}
impl Log {
    pub fn new(text: String, author: UserId) -> Self {
        Self {
            id: EntryId::new(),
            meta: Metadata {
                revision: Revision,
                version: Version(0),
                author,
                created: Date::now(),
            },
            text,
        }
    }
}
