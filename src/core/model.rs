use std::fmt::Display;

#[macro_export]
macro_rules! create_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy)]
        #[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
        #[cfg_attr(feature = "sqlx", sqlx(transparent))]
        pub struct $name(uuid::Uuid);

        #[cfg(feature = "sqlx")]
        impl From<$name> for u128 {
            fn from(value: $name) -> Self {
                value.0.as_u128()
            }
        }

        impl $name {
            pub fn create() -> Self {
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
pub struct Revision(u16);
#[derive(Debug)]
pub struct Date(u64);

#[derive(Debug)]
pub struct Username(String);
impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug)]
pub struct Author {
    id: UserId,
    name: Username,
}
impl Author {
    pub fn create(name: Username) -> Self {
        Self {
            id: UserId::create(),
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
    content: Text,
}
#[derive(Debug)]
pub struct Text(String);
