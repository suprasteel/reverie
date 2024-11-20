use std::{
    str::FromStr,
    time::{SystemTime, UNIX_EPOCH},
};

use derive_more::derive::{Display, Error};

#[macro_export]
macro_rules! create_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy)]
        #[cfg_attr(feature = "sqlx", derive(sqlx::Type, sqlx::FromRow))]
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
        impl FromStr for $name {
            type Err = ModelError;
            fn from_str(input: &str) -> Result<Self, Self::Err> {
                uuid::Uuid::parse_str(input)
                    .map(Self)
                    .or(Err(ModelError::InvalidId))
            }
        }
    };
}

#[derive(Debug, Display, Clone, Copy, Error)]
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

#[derive(Debug, Display, Error)]
#[display("invalid username {} (reason: {})", 0, 1)]
pub struct InvalidUsername(pub String, pub &'static str);
impl From<(&str, &'static str)> for InvalidUsername {
    fn from((username, reason): (&str, &'static str)) -> Self {
        Self(username.to_string(), reason)
    }
}
#[derive(Debug, Clone, Display)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Decode, sqlx::Encode, sqlx::FromRow))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
pub struct Username(String);
#[cfg(feature = "sqlx")]
impl sqlx::Type<sqlx::Sqlite> for Username {
    fn type_info() -> <sqlx::Sqlite as sqlx::Database>::TypeInfo {
        <String as sqlx::Type<sqlx::Sqlite>>::type_info()
    }
}
impl FromStr for Username {
    type Err = InvalidUsername;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "me" {
            return Ok(Self("me".to_string()));
        }
        let chars_count = s.chars().count();
        if chars_count < 6 {
            Err((s, "too short").into())
        } else if 24 < chars_count {
            Err((s, "too long").into())
        } else {
            Ok(Self(s.to_string()))
        }
    }
}
#[derive(Debug, Display)]
#[display("invalid project name {} (reason: {})", 0, 1)]
pub struct InvalidProjectName(pub String, pub &'static str);
impl std::error::Error for InvalidProjectName {}
impl From<(&str, &'static str)> for InvalidProjectName {
    fn from((project_name, reason): (&str, &'static str)) -> Self {
        Self(project_name.to_string(), reason)
    }
}
#[cfg_attr(feature = "sqlx", derive(sqlx::Type))]
#[cfg_attr(feature = "sqlx", sqlx(transparent))]
#[derive(Debug, Display, Clone)]
pub struct ProjectName(String);
impl FromStr for ProjectName {
    type Err = InvalidProjectName;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let chars_count = s.chars().count();
        if chars_count < 3 {
            Err((s, "too short").into())
        } else if 64 < chars_count {
            Err((s, "too long").into())
        } else {
            Ok(Self(s.to_string()))
        }
    }
}
#[derive(Debug, Clone, Display)]
#[display("#{id}-{name}")]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type, sqlx::FromRow, sqlx::Encode))]
pub struct User {
    id: UserId,
    pub(crate) name: Username,
}
// TODO: REMOVE
impl From<(uuid::Uuid, String)> for User {
    fn from((id, name): (uuid::Uuid, String)) -> Self {
        Self {
            id: UserId(id),
            name: Username(name),
        }
    }
}
impl From<([u8; 16], String)> for User {
    fn from((id, name): ([u8; 16], String)) -> Self {
        Self {
            id: UserId(uuid::Uuid::from_bytes(id)),
            name: Username(name),
        }
    }
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
#[derive(Debug, Display)]
#[display("Project #{id} - {name} ({})", meta.author)]
#[cfg_attr(feature = "sqlx", derive(sqlx::Type, sqlx::FromRow, sqlx::Encode))]
pub struct Project {
    id: ProjectId,
    #[sqlx(flatten)]
    pub(crate) meta: Metadata,
    pub(crate) name: ProjectName,
}
impl Project {
    pub fn new(name: ProjectName, author: UserId) -> Self {
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
    pub fn name(&self) -> &ProjectName {
        &self.name
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
#[derive(Debug, Clone, Display)]
#[display("#{id} - by {} - {text}\n", meta.author)]
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
