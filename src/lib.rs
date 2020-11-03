extern crate base64;
extern crate serde;

use lzma;
use serde_json;
use std::error;
use std::str;
use std::fmt::{self, Debug, Display};

#[derive(Debug)]
pub enum JURLError {
    DecodingError,
    CompressionError,
    DecompressionError,
    DeserializationError,
    SerializationError,
    UTF8Error
}

impl Display for JURLError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self)
    }
}

impl error::Error for JURLError {
    fn description(&self) -> &str {
        "Error from json-url"
    }

    fn cause(&self) -> Option<&dyn error::Error> { None }
}

pub fn pack<T>(content: &T) -> Result<String, JURLError>
where T: serde::Serialize
{
    let json = serde_json::to_string(&content);

    let compressed = match json {
        Ok(json_string) => lzma::compress(json_string.as_bytes(), 6),
        Err(_) => return Err(JURLError::SerializationError),
    };

    return match compressed {
        Ok(compressed_json) => Ok(base64::encode_config(compressed_json, base64::URL_SAFE_NO_PAD)),
        Err(_) => Err(JURLError::CompressionError)
    }
}

pub fn unpack<T>(packed: String) -> Result<T, JURLError>
where for <'de> T: serde::de::Deserialize<'de>
{
    let decode = base64::decode_config(packed, base64::URL_SAFE_NO_PAD);
    
    return match decode {
        Ok(d) => match lzma::decompress(d.as_slice()) {
            Ok(d) => match str::from_utf8(d.as_slice()) {
                Ok(s) => match serde_json::from_str::<T>(s) {
                    Ok(result) => Ok(result),
                    Err(_) => Err(JURLError::DeserializationError)
                },
                Err(_) => Err(JURLError::UTF8Error)
            },
            Err(_) => Err(JURLError::DecompressionError)
        },
        Err(_) => Err(JURLError::DecodingError)
    };
}