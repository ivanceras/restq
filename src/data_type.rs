use crate::{
    parser::ident,
    Error,
};
use pom::parser::*;
use sql_ast::ast as sql;
use std::{
    collections::BTreeMap,
    fmt,
    iter::FromIterator,
};

/// restq supports comprehensive data types
/// based on rust and postgresql type, combined together
/// format: <data_type>[?](contraint)
///     ?  - indicates it is optional, nullable in database context
/// example:
///     text? - nullable text
///     text(8..) - text with at least 8 characters long
///     text(..255) - text must not be more than 255 characters long.
///     u32(1) - u32 with default value of 1
///     u32(>10) - check value should be greater than 10
///     u32(10<column<=20) - check the value should be greater than 10 and less than or equal to 20
///     u32(<discount) - check value should be lesser than `discount` column
///     f32(0.0) - f32 with 0.0 as the default value
#[derive(PartialEq, Debug, Clone)]
pub enum DataType {
    /// bool
    Bool,
    /// 8 bit serial integer
    S8,
    /// 16 bit serial integer
    S16,
    /// 32 bit serial integer
    S32,
    /// 64 bit serial integer
    S64,
    /// f32
    F32,
    /// f64
    F64,
    /// u8
    U8,
    /// u16
    U16,
    /// u32
    U32,
    /// u64
    U64,
    /// i8
    I8,
    /// i16
    I16,
    /// i32
    I32,
    /// i64
    I64,
    /// Uuid, no default specified
    Uuid,
    /// Uuid with random as the default
    UuidRand,
    /// create a new uuid and generate a url friendly base64 using blob_uuid
    UuidSlug,
    /// local time with now as the default
    Local,
    /// Utc time with now as the default
    Utc,
    /// text/strings, generic text, no interpretation
    Text,
    /// A valid identifier string defined by begining of alpha_or_underscore character and
    /// optionally followed by alphnumeric characters
    Ident,
    /// A valid email address
    Email,
    /// A valid domain name
    Domain,
    /// A valid ip address
    IpAddr,
    /// A valid url
    Url,
}

impl DataType {
    ///returns all the supported data types
    pub fn all() -> Vec<DataType> {
        vec![
            DataType::Bool,
            DataType::S8,
            DataType::S16,
            DataType::S32,
            DataType::S64,
            DataType::F32,
            DataType::F64,
            DataType::U8,
            DataType::U16,
            DataType::U32,
            DataType::U64,
            DataType::I8,
            DataType::I16,
            DataType::I32,
            DataType::I64,
            DataType::Uuid,
            DataType::UuidRand,
            DataType::UuidSlug,
            DataType::Local,
            DataType::Utc,
            DataType::Text,
            DataType::Ident,
            DataType::Email,
            DataType::Domain,
            DataType::IpAddr,
            DataType::Url,
        ]
    }

    fn match_data_type(dt: &str) -> Result<Self, Error> {
        match dt {
            "bool" => Ok(DataType::Bool),
            "s8" => Ok(DataType::S8),
            "s16" => Ok(DataType::S16),
            "s32" => Ok(DataType::S32),
            "s64" => Ok(DataType::S64),
            "u8" => Ok(DataType::U8),
            "u16" => Ok(DataType::U16),
            "u32" => Ok(DataType::U32),
            "u64" => Ok(DataType::U64),
            "i8" => Ok(DataType::I8),
            "i16" => Ok(DataType::I16),
            "i32" => Ok(DataType::I32),
            "i64" => Ok(DataType::I64),
            "f32" => Ok(DataType::F32),
            "f64" => Ok(DataType::F64),
            "uuid" => Ok(DataType::Uuid),
            "uuid_rand" => Ok(DataType::UuidRand),
            "uuid_slug" => Ok(DataType::UuidSlug),
            "local" => Ok(DataType::Local),
            "utc" => Ok(DataType::Utc),
            "text" => Ok(DataType::Text),
            "ident" => Ok(DataType::Ident),
            "email" => Ok(DataType::Email),
            "domain" => Ok(DataType::Domain),
            "ip_addr" => Ok(DataType::IpAddr),
            "url" => Ok(DataType::Url),
            _ => Err(Error::InvalidDataType(dt.to_string())),
        }
    }
}

impl fmt::Display for DataType {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let display = match self {
            DataType::Bool => "bool",
            DataType::S8 => "s8",
            DataType::S16 => "s16",
            DataType::S32 => "s32",
            DataType::S64 => "s64",
            DataType::F32 => "f32",
            DataType::F64 => "f64",
            DataType::U8 => "u8",
            DataType::U16 => "u16",
            DataType::U32 => "u32",
            DataType::U64 => "u64",
            DataType::I8 => "i8",
            DataType::I16 => "i16",
            DataType::I32 => "i32",
            DataType::I64 => "i64",
            DataType::Uuid => "uuid",
            DataType::UuidRand => "uuid_rand",
            DataType::UuidSlug => "uuid_slug",
            DataType::Local => "local",
            DataType::Utc => "utc",
            DataType::Text => "text",
            DataType::Ident => "ident",
            DataType::Email => "email",
            DataType::Domain => "domain",
            DataType::IpAddr => "ip_addr",
            DataType::Url => "url",
        };

        write!(f, "{}", display)
    }
}

pub fn data_type<'a>() -> Parser<'a, char, DataType> {
    ident().convert(|v| DataType::match_data_type(&v))
}

impl Into<sql::DataType> for &DataType {
    fn into(self) -> sql::DataType {
        match self {
            DataType::Bool => sql::DataType::Boolean,
            DataType::S8 => sql::DataType::SmallInt,
            DataType::S16 => sql::DataType::SmallInt,
            DataType::S32 => sql::DataType::Int,
            DataType::S64 => sql::DataType::BigInt,
            DataType::F32 => sql::DataType::Float(None),
            DataType::F64 => sql::DataType::Float(None),
            DataType::U8 => sql::DataType::SmallInt,
            DataType::U16 => sql::DataType::SmallInt,
            DataType::U32 => sql::DataType::Int,
            DataType::U64 => sql::DataType::BigInt,
            DataType::I8 => sql::DataType::SmallInt,
            DataType::I16 => sql::DataType::SmallInt,
            DataType::I32 => sql::DataType::Int,
            DataType::I64 => sql::DataType::BigInt,
            DataType::Uuid => sql::DataType::Uuid,
            DataType::UuidRand => sql::DataType::Uuid,
            DataType::UuidSlug => sql::DataType::Text,
            DataType::Local => sql::DataType::Timestamp,
            DataType::Utc => sql::DataType::Timestamp,
            DataType::Text => sql::DataType::Text,
            DataType::Ident => sql::DataType::Text,
            DataType::Email => sql::DataType::Text,
            DataType::Domain => sql::DataType::Text,
            DataType::IpAddr => sql::DataType::Text,
            DataType::Url => sql::DataType::Text,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::utils::*;

    #[test]
    fn test_data_type() {
        let input = to_chars("s32");
        let ret = data_type().parse(&input).expect("must be parsed");
        println!("{:#?}", ret);
        assert_eq!(ret, DataType::S32);
    }

    #[test]
    fn test_invalid_data_type() {
        let input = to_chars("x32");
        let ret = data_type().parse(&input);
        println!("{:#?}", ret);
        assert!(ret.is_err());
    }

    #[test]
    fn test_invalid_more_data_type() {
        let input = to_chars("serial32");
        let ret = data_type().parse(&input);
        println!("{:#?}", ret);
        assert!(ret.is_err());
        let err = ret.err().unwrap();
        println!("{}", err);
        assert!(err.to_string().contains(r#"InvalidDataType("serial32")"#))
    }

    #[test]
    fn all_data_types() {
        let all = [
            "bool",
            "s8",
            "s16",
            "s32",
            "s64",
            "u8",
            "u16",
            "u32",
            "u64",
            "i8",
            "i16",
            "i32",
            "i64",
            "f32",
            "f64",
            "uuid",
            "uuid_rand",
            "uuid_slug",
            "local",
            "utc",
            "text",
            "ident",
            "email",
            "domain",
            "ip_addr",
            "url",
        ];

        for d in all.iter() {
            println!("trying {}...", d);
            let input = to_chars(d);
            let ret = data_type().parse(&input).expect("must be parsed");
            println!("{} = {:#?}", d, ret);
        }
    }
}
