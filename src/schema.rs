use cosmwasm_schema::cw_serde;

use crate::error::ContractError;

pub const DEFAULT_PADDING_STRING: u16 = 1024;
pub const DEFAULT_PADDING_ARRAY: u16 = 1024;
pub const DEFAULT_PADDING_OBJECT: u16 = 1024;

#[cw_serde]
pub struct EntitySchema {
    pub name: String,
    pub properties: Vec<EntityProperty>,
}

#[cw_serde]
pub struct EntityProperty {
    pub indexed: Option<bool>,
    pub required: Option<bool>,
    pub value: EntityPropertyParams,
    pub name: String,
}

#[cw_serde]
pub enum EntityPropertyParams {
    Array { max_byte_size: Option<u16> },
    Object { max_byte_size: Option<u16> },
    String { max_byte_size: Option<u16> },
    U8 {},
    U16 {},
    U32 {},
    U64 {},
    U128 {},
    I8 {},
    I16 {},
    I32 {},
    I64 {},
    I128 {},
    Bool {},
}

impl EntitySchema {
    // pub fn validate(
    //     &self,
    //     entity: serde_json::Value,
    // ) {
    //     todo!()
    // }
}

impl EntityProperty {
    pub fn to_bytes(
        &self,
        value: &serde_json::Value,
    ) -> Result<Vec<u8>, ContractError> {
        // NOTE: This assumes that we've already validated.
        Ok(match self.value {
            EntityPropertyParams::String { max_byte_size } => {
                let padding = max_byte_size.unwrap_or(DEFAULT_PADDING_STRING) as usize;
                self.pad(value.as_str().unwrap().as_bytes().to_vec(), padding)?
            },
            EntityPropertyParams::Array { max_byte_size } => {
                let padding = max_byte_size.unwrap_or(DEFAULT_PADDING_ARRAY) as usize;
                self.pad(value.to_string().as_bytes().to_vec(), padding)?
            },
            EntityPropertyParams::Object { max_byte_size } => {
                let padding = max_byte_size.unwrap_or(DEFAULT_PADDING_OBJECT) as usize;
                self.pad(value.to_string().as_bytes().to_vec(), padding)?
            },
            EntityPropertyParams::U8 {} => value.as_u64().unwrap().clamp(0, u8::MAX as u64).to_le_bytes()[..1].to_vec(),
            EntityPropertyParams::U16 {} => {
                value.as_u64().unwrap().clamp(0, u16::MAX as u64).to_le_bytes()[..2].to_vec()
            },
            EntityPropertyParams::U32 {} => {
                value.as_u64().unwrap().clamp(0, u32::MAX as u64).to_le_bytes()[..4].to_vec()
            },
            EntityPropertyParams::U64 {} => value.as_u64().unwrap().clamp(0, u64::MAX).to_le_bytes()[..8].to_vec(),
            EntityPropertyParams::U128 {} => self.pad(value.as_str().unwrap().as_bytes().to_vec(), 16)?,
            EntityPropertyParams::I8 {} => value.as_i64().unwrap().clamp(0, i8::MAX as i64).to_le_bytes()[..1].to_vec(),
            EntityPropertyParams::I16 {} => {
                value.as_i64().unwrap().clamp(0, i16::MAX as i64).to_le_bytes()[..2].to_vec()
            },
            EntityPropertyParams::I32 {} => {
                value.as_i64().unwrap().clamp(0, i32::MAX as i64).to_le_bytes()[..4].to_vec()
            },
            EntityPropertyParams::I64 {} => value.as_i64().unwrap().clamp(0, i64::MAX).to_le_bytes()[..8].to_vec(),
            EntityPropertyParams::I128 {} => self.pad(value.as_str().unwrap().as_bytes().to_vec(), 16)?,
            EntityPropertyParams::Bool {} => vec![if value.as_bool().unwrap() { 1u8 } else { 0u8 }],
        })
    }

    // fn unpad(
    //     &self,
    //     bytes: Vec<u8>,
    // ) -> Vec<u8> {
    //     let len = bytes.len();
    //     let mut i = len - 1;
    //     let mut bytes = bytes;
    //     while i != 0 && bytes[i] == 0 {
    //         bytes.pop();
    //         i -= 1;
    //     }
    //     bytes
    // }

    fn pad(
        &self,
        vec: Vec<u8>,
        target_length: usize,
    ) -> Result<Vec<u8>, ContractError> {
        let n = target_length.saturating_sub(vec.len());
        let padded_vec = {
            if n > 0 {
                let mut padded_vec = vec;
                padded_vec.reserve(n);
                for _ in 0..n {
                    padded_vec.push(0)
                }
                padded_vec
            } else {
                vec
            }
        };
        if padded_vec.len() > target_length {
            return Err(self.validation_error("beyond allowed max byte size").unwrap_err());
        }
        Ok(padded_vec)
    }

    pub fn validate(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), ContractError> {
        match self.value {
            EntityPropertyParams::String { max_byte_size } => {
                self.validate_string(value, max_byte_size)?;
            },
            EntityPropertyParams::Object { max_byte_size } => {
                self.validate_object(value, max_byte_size)?;
            },
            EntityPropertyParams::Array { max_byte_size } => {
                self.validate_array(value, max_byte_size)?;
            },
            EntityPropertyParams::Bool {} => {
                self.validate_bool(value)?;
            },
            EntityPropertyParams::U8 {} => {
                self.validate_number(value, u8::MIN.into(), u8::MAX.into())?;
            },
            EntityPropertyParams::U16 {} => {
                self.validate_number(value, u16::MIN.into(), u16::MAX.into())?;
            },
            EntityPropertyParams::U32 {} => {
                self.validate_number(value, u32::MIN.into(), u32::MAX.into())?;
            },
            EntityPropertyParams::U64 {} => {
                self.validate_number(value, u64::MIN.into(), u64::MAX.into())?;
            },
            EntityPropertyParams::U128 {} => {
                self.validate_u128(value, u128::MIN, u128::MAX)?;
            },
            EntityPropertyParams::I8 {} => {
                self.validate_number(value, i8::MIN.into(), i8::MAX.into())?;
            },
            EntityPropertyParams::I16 {} => {
                self.validate_number(value, i16::MIN.into(), i16::MAX.into())?;
            },
            EntityPropertyParams::I32 {} => {
                self.validate_number(value, i32::MIN.into(), i32::MAX.into())?;
            },
            EntityPropertyParams::I64 {} => {
                self.validate_number(value, i64::MIN.into(), i64::MAX.into())?;
            },
            EntityPropertyParams::I128 {} => {
                self.validate_number(value, i128::MIN.into(), i128::MAX.into())?;
            },
        }
        Ok(())
    }

    fn validation_error(
        &self,
        reason: &str,
    ) -> Result<(), ContractError> {
        Err(ContractError::ValidationError {
            reason: format!("{} - {}", self.name, reason),
        })
    }

    fn validate_string(
        &self,
        value: &serde_json::Value,
        max_byte_size: Option<u16>,
    ) -> Result<(), ContractError> {
        if !value.is_string() {
            return self.validation_error("expected string");
        }
        // Check size in bytes relative to capacity, raising error as side-effect
        let padding = max_byte_size.unwrap_or(DEFAULT_PADDING_STRING) as usize;
        self.pad(value.to_string().as_bytes().to_vec(), padding)?;
        Ok(())
    }

    fn validate_object(
        &self,
        value: &serde_json::Value,
        max_byte_size: Option<u16>,
    ) -> Result<(), ContractError> {
        if !value.is_object() {
            return self.validation_error("expected object");
        }
        // Check size in bytes relative to capacity, raising error as side-effect
        let padding = max_byte_size.unwrap_or(DEFAULT_PADDING_OBJECT) as usize;
        self.pad(value.to_string().as_bytes().to_vec(), padding)?;
        Ok(())
    }

    fn validate_array(
        &self,
        value: &serde_json::Value,
        max_byte_size: Option<u16>,
    ) -> Result<(), ContractError> {
        if !value.is_array() {
            return self.validation_error("expected array");
        }
        // Check size in bytes relative to capacity, raising error as side-effect
        let padding = max_byte_size.unwrap_or(DEFAULT_PADDING_ARRAY) as usize;
        self.pad(value.to_string().as_bytes().to_vec(), padding)?;
        Ok(())
    }

    fn validate_bool(
        &self,
        value: &serde_json::Value,
    ) -> Result<(), ContractError> {
        if !value.is_boolean() {
            return self.validation_error("expected boolean");
        }
        Ok(())
    }

    fn validate_u128(
        &self,
        value: &serde_json::Value,
        min: u128,
        max: u128,
    ) -> Result<(), ContractError> {
        if let Some(s) = value.as_str() {
            if let Ok(x) = u128::from_str_radix(s, 10) {
                if x > max {
                    return self.validation_error("too large");
                }
                if x < min {
                    return self.validation_error("too small");
                }
            } else {
                return self.validation_error("invalid u128");
            }
        } else {
            return self.validation_error("invalid u128");
        }
        Ok(())
    }

    fn validate_number(
        &self,
        value: &serde_json::Value,
        min: i128,
        max: i128,
    ) -> Result<(), ContractError> {
        if let Some(x) = value.as_u64() {
            if (x as i128) > max {
                return self.validation_error("too large");
            }
        } else if let Some(x) = value.as_i64() {
            if (x as i128) > max {
                return self.validation_error("too large");
            }
            if (x as i128) < min {
                return self.validation_error("too small");
            }
        } else {
            return self.validation_error("invalid type");
        }
        Ok(())
    }
}
