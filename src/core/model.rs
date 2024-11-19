use std::{
    fmt::Display,
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

#[macro_export]
macro_rules! create_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy)]
        #[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
        #[cfg_attr(feature = "sqlx", sqlx(transparent))]
        pub struct $name(sqlx::types::Uuid);

        impl std::fmt::Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0.to_string())
            }
        }

        impl Default for $name {
            fn default() -> Self {
                Self(uuid::Uuid::now_v7())
            }
        }
        impl $name {
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

#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct Version(u16);
/// hardcoded in lib
pub type Revision = i16; // make static string
#[derive(Debug, Copy, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct Date(i64);
impl Date {
    pub fn now() -> Self {
        Self(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos() as i64,
        )
    }
    pub fn as_i64(&self) -> i64 {
        self.0
    }

    fn _iso8601(&self) -> String {
        // https://stackoverflow.com/questions/64146345/how-do-i-convert-a-systemtime-to-iso-8601-in-rust
        todo!()
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct Username(String);
impl FromStr for Username {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "me" {
            return Ok(Self("me".to_string()));
        }
        let chars_count = s.chars().count();
        if 6 < chars_count && chars_count < 24 {
            Ok(Self(s.to_string()))
        } else {
            Err("6 < project name < 24")
        }
    }
}
impl Display for Username {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type, sqlx::FromRow, sqlx::Encode))]
pub struct User {
    id: UserId,
    pub(crate) name: Username,
}
impl User {
    pub fn create(name: Username) -> Self {
        Self {
            id: UserId::default(),
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
#[cfg_attr(feature = "sqlx", derive(sqlx::Type, sqlx::FromRow, sqlx::Encode))]
pub struct Project {
    id: ProjectId,
    #[sqlx(flatten)]
    pub(crate) meta: Metadata,
    pub(crate) name: String,
}
impl Project {
    pub fn new(name: String, author: UserId) -> Self {
        Self {
            id: ProjectId::default(),
            meta: Metadata {
                revision: 0,
                version: Version(0),
                author,
                created: Date::now(),
            },
            name,
        }
    }
    pub fn id(&self) -> ProjectId {
        self.id
    }
    pub fn name(&self) -> &str {
        self.name.as_str()
    }
}

#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type, sqlx::FromRow))]
pub struct Metadata {
    pub(crate) revision: Revision,
    pub(crate) version: Version,
    pub(crate) author: UserId,
    pub(crate) created: Date,
}
impl Metadata {
    pub fn new(user: UserId) -> Self {
        Self {
            revision: 0,
            version: Version(0),
            author: user,
            created: Date::now(),
        }
    }
}
#[derive(Debug, Clone)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type, sqlx::FromRow))]
pub struct Log {
    id: EntryId,
    #[sqlx(flatten)]
    pub(crate) meta: Metadata,
    pub(crate) text: String,
}
impl Log {
    pub fn new(text: String, author: UserId) -> Self {
        Self {
            id: EntryId::default(),
            meta: Metadata {
                revision: 0,
                version: Version(0),
                author,
                created: Date::now(),
            },
            text,
        }
    }
    pub fn id(&self) -> EntryId {
        self.id
    }
}
