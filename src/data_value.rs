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
    NaiveDate,
    NaiveDateTime,
    NaiveTime,
    Utc,
};
use sql_ast::ast as sql;
use std::fmt;
use url::Url;
use uuid::Uuid;

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
    Url(Url),
    Bytes(Vec<u8>),
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
            DataValue::Url(_) => DataType::Url,
            DataValue::Bytes(_) => DataType::Bytes,
        };
        Some(dt)
    }
}

fn naive_date_parser(v: &str) -> NaiveDateTime {
    if let Ok(dt) = DateTime::parse_from_rfc3339(v) {
        dt.naive_local()
    } else if let Ok(ts) =
        NaiveDateTime::parse_from_str(&v, "%Y-%m-%dT%H:%M:%S%z")
    {
        ts
    } else if let Ok(ts) =
        NaiveDateTime::parse_from_str(&v, "%Y-%m-%d %H:%M:%S")
    {
        ts
    } else if let Ok(ts) =
        NaiveDateTime::parse_from_str(&v, "%Y-%m-%d %H:%M:%S.%f")
    {
        ts
    } else if let Ok(nd) = NaiveDate::parse_from_str(&v, "%Y-%m-%d") {
        NaiveDateTime::new(nd, NaiveTime::from_hms_milli(0, 0, 0, 0))
    } else {
        panic!("unable to parse timestamp: {:?}", v);
    }
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

impl Into<Value> for &DataValue {
    fn into(self) -> Value {
        match self {
            DataValue::Bool(v) => Value::Bool(*v),
            DataValue::U8(v) => Value::Number(*v as f64),
            DataValue::U16(v) => Value::Number(*v as f64),
            DataValue::U32(v) => Value::Number(*v as f64),
            DataValue::U64(v) => Value::Number(*v as f64),
            DataValue::I8(v) => Value::Number(*v as f64),
            DataValue::I16(v) => Value::Number(*v as f64),
            DataValue::I32(v) => Value::Number(*v as f64),
            DataValue::I64(v) => Value::Number(*v as f64),
            DataValue::F32(v) => Value::Number(*v as f64),
            DataValue::F64(v) => Value::Number(*v as f64),
            DataValue::Text(v) => Value::String(v.clone()),
            DataValue::Local(v) => Value::String(v.to_rfc3339()),
            _ => panic!("todo for: {:?}", self),
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
                    DataType::S8 => DataValue::S8(if v { 1 } else { 0 }),
                    DataType::S16 => DataValue::S16(if v { 1 } else { 0 }),
                    DataType::S32 => DataValue::S32(if v { 1 } else { 0 }),
                    DataType::S64 => DataValue::S64(if v { 1 } else { 0 }),
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
                    DataType::S8 => DataValue::S8(v as u8),
                    DataType::S16 => DataValue::S16(v as u16),
                    DataType::S32 => DataValue::S32(v as u32),
                    DataType::S64 => DataValue::S64(v as u64),
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
                        if v.is_empty() {
                            DataValue::Nil
                        } else if let Ok(v) = v.parse::<u8>() {
                            DataValue::S8(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::S16 => {
                        if v.is_empty() {
                            DataValue::Nil
                        } else if let Ok(v) = v.parse::<u16>() {
                            DataValue::S16(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::S32 => {
                        if v.is_empty() {
                            DataValue::Nil
                        } else if let Ok(v) = v.parse::<u32>() {
                            DataValue::S32(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::S64 => {
                        if v.is_empty() {
                            DataValue::Nil
                        } else if let Ok(v) = v.parse::<u64>() {
                            DataValue::S64(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::U8 => {
                        if v.is_empty() {
                            DataValue::Nil
                        } else if v.is_empty() {
                            DataValue::Nil
                        } else if let Ok(v) = v.parse::<u8>() {
                            DataValue::U8(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::U16 => {
                        if v.is_empty() {
                            DataValue::Nil
                        } else if let Ok(v) = v.parse::<u16>() {
                            DataValue::U16(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::U32 => {
                        if v.is_empty() {
                            DataValue::Nil
                        } else if let Ok(v) = v.parse::<u32>() {
                            DataValue::U32(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::U64 => {
                        if v.is_empty() {
                            DataValue::Nil
                        } else if let Ok(v) = v.parse::<u64>() {
                            DataValue::U64(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::I8 => {
                        if v.is_empty() {
                            DataValue::Nil
                        } else if let Ok(v) = v.parse::<i8>() {
                            DataValue::I8(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::I16 => {
                        if v.is_empty() {
                            DataValue::Nil
                        } else if let Ok(v) = v.parse::<i16>() {
                            DataValue::I16(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::I32 => {
                        if v.is_empty() {
                            DataValue::Nil
                        } else if let Ok(v) = v.parse::<i32>() {
                            DataValue::I32(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::I64 => {
                        if v.is_empty() {
                            DataValue::Nil
                        } else if let Ok(v) = v.parse::<i64>() {
                            DataValue::I64(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::F32 => {
                        if v.is_empty() {
                            DataValue::Nil
                        } else if let Ok(v) = v.parse::<f32>() {
                            DataValue::F32(v)
                        } else {
                            panic!(
                                "unsupported conversion from {:?} to {:?}",
                                value, required_type
                            );
                        }
                    }
                    DataType::F64 => {
                        if v.is_empty() {
                            DataValue::Nil
                        } else if let Ok(v) = v.parse::<f64>() {
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
                    DataType::Bytes => {
                        let bytes = base64::decode_config(&v, base64::MIME)
                            .expect("must be a valid base64 bytes");
                        DataValue::Bytes(bytes)
                    }
                    DataType::Json => DataValue::Text(v.to_string()),
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

impl fmt::Display for DataValue {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            DataValue::Nil => write!(f, ""),
            DataValue::Bool(v) => write!(f, "{}", v),
            DataValue::S8(v) => write!(f, "{}", v),
            DataValue::S16(v) => write!(f, "{}", v),
            DataValue::S32(v) => write!(f, "{}", v),
            DataValue::S64(v) => write!(f, "{}", v),
            DataValue::F32(v) => write!(f, "{}", v),
            DataValue::F64(v) => write!(f, "{}", v),
            DataValue::U8(v) => write!(f, "{}", v),
            DataValue::U16(v) => write!(f, "{}", v),
            DataValue::U32(v) => write!(f, "{}", v),
            DataValue::U64(v) => write!(f, "{}", v),
            DataValue::I8(v) => write!(f, "{}", v),
            DataValue::I16(v) => write!(f, "{}", v),
            DataValue::I32(v) => write!(f, "{}", v),
            DataValue::I64(v) => write!(f, "{}", v),
            DataValue::Uuid(v) => write!(f, "{}", v),
            DataValue::UuidRand(v) => write!(f, "{}", v),
            DataValue::UuidSlug(v) => write!(f, "{}", v),
            DataValue::Local(v) => write!(f, "{}", v.to_rfc3339()),
            DataValue::Utc(v) => write!(f, "{}", v.to_rfc3339()),
            DataValue::Text(v) => write!(f, "{}", v),
            DataValue::Ident(v) => write!(f, "{}", v),
            DataValue::Url(v) => write!(f, "{}", v),
            DataValue::Bytes(v) => {
                let encoded = base64::encode_config(&v, base64::MIME);
                write!(f, "{}", encoded)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn parse_simple_date() {
        let date = "2006-02-14";

        let parsed = NaiveDateTime::parse_from_str(date, "%Y-%m-%d");
        println!("parsed: {:?}", parsed);

        let res = naive_date_parser(date);
        let naive_date = NaiveDate::from_ymd(2006, 2, 14);
        let naive_time = NaiveTime::from_hms_milli(0, 0, 0, 0);
        assert_eq!(res, NaiveDateTime::new(naive_date, naive_time));
    }

    #[test]
    fn parse_dates() {
        let date = "2006-02-15T09:34:33+00:00";
        let res = naive_date_parser(date);
        println!("res: {}", res);
        let naive_date = NaiveDate::from_ymd(2006, 2, 15);
        let naive_time = NaiveTime::from_hms_milli(9, 34, 33, 0);
        assert_eq!(res, NaiveDateTime::new(naive_date, naive_time));
    }
}
