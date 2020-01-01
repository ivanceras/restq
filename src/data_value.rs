use crate::{
    ast::{
        Expr,
        Value,
    },
    data_type::DataType,
};
use chrono::{
    offset::FixedOffset,
    DateTime,
    Local,
    NaiveDateTime,
    Utc,
};
use sqlparser::ast as sql;
use std::net::IpAddr;
use url::Url;
use uuid::Uuid;
use webpki::DNSName;

/// strict data value
/// where each has exact byte definitions, etc.
#[derive(PartialEq, Debug, Clone)]
pub enum DataValue {
    Nil,
    Bool(bool),
    S8(u8),
    S16(u16),
    S32(u32),
    S64(u64),
    F32(f32),
    F64(f64),
    U8(u8),
    U16(u16),
    U32(u32),
    U64(u64),
    I8(i8),
    I16(i16),
    I32(i32),
    I64(i64),
    Uuid(Uuid),
    UuidRand(Uuid),
    UuidSlug(String),
    Local(DateTime<Local>),
    Utc(DateTime<Utc>),
    Text(String),
    Ident(String),
    Email(String),
    Domain(DNSName),
    IpAddr(IpAddr),
    Url(Url),
}

impl DataValue {
    pub fn get_data_type(&self) -> Option<DataType> {
        let dt = match self {
            DataValue::Nil => {
                return None;
            }
            DataValue::Bool(_) => DataType::Bool,
            DataValue::S8(_) => DataType::S8,
            DataValue::S16(_) => DataType::S16,
            DataValue::S32(_) => DataType::S32,
            DataValue::S64(_) => DataType::S64,
            DataValue::F32(_) => DataType::F32,
            DataValue::F64(_) => DataType::F64,
            DataValue::U8(_) => DataType::U8,
            DataValue::U16(_) => DataType::U16,
            DataValue::U32(_) => DataType::U32,
            DataValue::U64(_) => DataType::U64,
            DataValue::I8(_) => DataType::I8,
            DataValue::I16(_) => DataType::I16,
            DataValue::I32(_) => DataType::I32,
            DataValue::I64(_) => DataType::I64,
            DataValue::Uuid(_) => DataType::Uuid,
            DataValue::UuidRand(_) => DataType::UuidRand,
            DataValue::UuidSlug(_) => DataType::UuidSlug,
            DataValue::Local(_) => DataType::Local,
            DataValue::Utc(_) => DataType::Utc,
            DataValue::Text(_) => DataType::Text,
            DataValue::Ident(_) => DataType::Ident,
            DataValue::Email(_) => DataType::Email,
            DataValue::Domain(_) => DataType::Domain,
            DataValue::IpAddr(_) => DataType::IpAddr,
            DataValue::Url(_) => DataType::Url,
        };
        Some(dt)
    }
}

fn naive_date_parser(v: &str) -> NaiveDateTime {
    let ts = NaiveDateTime::parse_from_str(&v, "%Y-%m-%d %H:%M:%S");
    let ts = if let Ok(ts) = ts {
        ts
    } else {
        let ts = NaiveDateTime::parse_from_str(&v, "%Y-%m-%d %H:%M:%S.%f");
        if let Ok(ts) = ts {
            ts
        } else {
            panic!("unable to parse timestamp: {}", v);
        }
    };
    ts
}

impl Into<u32> for DataValue {
    fn into(self) -> u32 {
        match self {
            DataValue::U8(v) => v as u32,
            DataValue::U16(v) => v as u32,
            DataValue::U32(v) => v,
            DataValue::U64(v) => v as u32,
            DataValue::I8(v) => v as u32,
            DataValue::I16(v) => v as u32,
            DataValue::I32(v) => v as u32,
            DataValue::I64(v) => v as u32,
            _ => {
                panic!(
                    "unsupported conversion: {:?} to u32",
                    self.get_data_type()
                )
            }
        }
    }
}

impl Into<u64> for DataValue {
    fn into(self) -> u64 {
        match self {
            DataValue::U8(v) => v as u64,
            DataValue::U16(v) => v as u64,
            DataValue::U32(v) => v as u64,
            DataValue::U64(v) => v,
            DataValue::I8(v) => v as u64,
            DataValue::I16(v) => v as u64,
            DataValue::I32(v) => v as u64,
            DataValue::I64(v) => v as u64,
            _ => {
                panic!(
                    "unsupported conversion: {:?} to u64",
                    self.get_data_type()
                )
            }
        }
    }
}

impl Into<f32> for DataValue {
    fn into(self) -> f32 {
        match self {
            DataValue::U8(v) => v as f32,
            DataValue::U16(v) => v as f32,
            DataValue::U32(v) => v as f32,
            DataValue::U64(v) => v as f32,
            DataValue::I8(v) => v as f32,
            DataValue::I16(v) => v as f32,
            DataValue::I32(v) => v as f32,
            DataValue::I64(v) => v as f32,
            DataValue::F32(v) => v,
            DataValue::F64(v) => v as f32,
            _ => {
                panic!(
                    "unsupported conversion: {:?} to f32",
                    self.get_data_type()
                )
            }
        }
    }
}

impl Into<String> for DataValue {
    fn into(self) -> String {
        match self {
            DataValue::Text(v) => v,
            _ => {
                panic!(
                    "unsupported conversion: {:?} to String",
                    self.get_data_type()
                )
            }
        }
    }
}

impl<'a> Into<&'a str> for &'a DataValue {
    fn into(self) -> &'a str {
        match self {
            DataValue::Text(ref v) => v,
            _ => {
                panic!(
                    "unsupported conversion: {:?} to &str",
                    self.get_data_type()
                )
            }
        }
    }
}

impl Into<Expr> for Value {
    fn into(self) -> Expr {
        Expr::Value(self)
    }
}

impl Into<sql::Expr> for &Value {
    fn into(self) -> sql::Expr {
        let expr: Expr = Into::into(self.clone());
        Into::into(&expr)
    }
}

impl Into<sql::Value> for &DataValue {
    fn into(self) -> sql::Value {
        match self {
            DataValue::Bool(v) => sql::Value::Boolean(*v),
            DataValue::U8(v) => sql::Value::Number(v.to_string()),
            DataValue::U16(v) => sql::Value::Number(v.to_string()),
            DataValue::U32(v) => sql::Value::Number(v.to_string()),
            DataValue::U64(v) => sql::Value::Number(v.to_string()),
            DataValue::F32(v) => sql::Value::Number(v.to_string()),
            DataValue::F64(v) => sql::Value::Number(v.to_string()),
            _ => todo!(),
        }
    }
}

impl Into<sql::Expr> for &DataValue {
    fn into(self) -> sql::Expr {
        sql::Expr::Value(Into::into(self))
    }
}

/// cast the value into DataValue hinted by the data_type
pub fn cast_data_value(value: &Value, required_type: &DataType) -> DataValue {
    if *value == Value::Null {
        DataValue::Nil
    } else {
        match *value {
            Value::Bool(v) => {
                match *required_type {
                    DataType::Bool => DataValue::Bool(v),
                    DataType::U8 => DataValue::U8(if v { 1 } else { 0 }),
                    DataType::U16 => DataValue::U16(if v { 1 } else { 0 }),
                    DataType::U32 => DataValue::U32(if v { 1 } else { 0 }),
                    DataType::U64 => DataValue::U64(if v { 1 } else { 0 }),
                    DataType::I8 => DataValue::I8(if v { 1 } else { 0 }),
                    DataType::I16 => DataValue::I16(if v { 1 } else { 0 }),
                    DataType::I32 => DataValue::I32(if v { 1 } else { 0 }),
                    DataType::I64 => DataValue::I64(if v { 1 } else { 0 }),
                    _ => {
                        panic!(
                            "unsupported conversion from {:?} to {:?}",
                            value, required_type
                        )
                    }
                }
            }
            Value::Number(v) => {
                match *required_type {
                    DataType::U8 => DataValue::U8(v as u8),
                    DataType::U16 => DataValue::U16(v as u16),
                    DataType::U32 => DataValue::U32(v as u32),
                    DataType::U64 => DataValue::U64(v as u64),
                    DataType::I8 => DataValue::I8(v as i8),
                    DataType::I16 => DataValue::I16(v as i16),
                    DataType::I32 => DataValue::I32(v as i32),
                    DataType::I64 => DataValue::I64(v as i64),
                    DataType::F32 => DataValue::F32(v as f32),
                    DataType::F64 => DataValue::F64(v as f64),
                    _ => {
                        panic!(
                            "unsupported conversion from {:?} to {:?}",
                            value, required_type
                        )
                    }
                }
            }
            Value::String(ref v) => {
                match *required_type {
                    DataType::Text => DataValue::Text(v.to_string()),
                    DataType::Bool => {
                        match v.as_ref() {
                            "true" => DataValue::Bool(true),
                            "false" => DataValue::Bool(false),
                            "1" => DataValue::Bool(true),
                            "0" => DataValue::Bool(false),
                            _ => DataValue::Bool(false),
                        }
                    }
                    DataType::S8 => {
                        if let Ok(v) = v.parse::<u8>() {
                            DataValue::S8(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::S16 => {
                        if let Ok(v) = v.parse::<u16>() {
                            DataValue::S16(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::S32 => {
                        if let Ok(v) = v.parse::<u32>() {
                            DataValue::S32(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::S64 => {
                        if let Ok(v) = v.parse::<u64>() {
                            DataValue::S64(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::U8 => {
                        if let Ok(v) = v.parse::<u8>() {
                            DataValue::U8(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::U16 => {
                        if let Ok(v) = v.parse::<u16>() {
                            DataValue::U16(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::U32 => {
                        if let Ok(v) = v.parse::<u32>() {
                            DataValue::U32(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::U64 => {
                        if let Ok(v) = v.parse::<u64>() {
                            DataValue::U64(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::I8 => {
                        if let Ok(v) = v.parse::<i8>() {
                            DataValue::I8(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::I16 => {
                        if let Ok(v) = v.parse::<i16>() {
                            DataValue::I16(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::I32 => {
                        if let Ok(v) = v.parse::<i32>() {
                            DataValue::I32(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::I64 => {
                        if let Ok(v) = v.parse::<i64>() {
                            DataValue::I64(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::F32 => {
                        if let Ok(v) = v.parse::<f32>() {
                            DataValue::F32(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::F64 => {
                        if let Ok(v) = v.parse::<f64>() {
                            DataValue::F64(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::Local => {
                        let ts = naive_date_parser(&v);
                        let local = DateTime::<Local>::from_utc(
                            ts,
                            FixedOffset::east(0),
                        );
                        DataValue::Local(local)
                    }
                    DataType::Utc => {
                        let ts = naive_date_parser(&v);
                        let utc = DateTime::<Utc>::from_utc(ts, Utc);
                        DataValue::Utc(utc)
                    }
                    DataType::Uuid | DataType::UuidRand => {
                        let uuid =
                            Uuid::parse_str(&v).expect("unable to parse uuid");
                        DataValue::Uuid(uuid)
                    }
                    DataType::UuidSlug => DataValue::UuidSlug(v.to_string()),
                    //TODO: validate identifier
                    DataType::Ident => DataValue::Ident(v.to_string()),
                    _ => {
                        panic!(
                            "unsupported conversion from {:?} to {:?}",
                            value, required_type
                        )
                    }
                }
            }
            _ => {
                panic!(
                    "unsupported conversion from {:?} to {:?}",
                    value, required_type
                )
            }
        }
    }
}
