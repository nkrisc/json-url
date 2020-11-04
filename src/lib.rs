extern crate base64;
extern crate serde;

use lzma;
use serde_json::{error::Category};
use std::error;
use std::str;
use std::fmt::{self, Debug, Display};

#[derive(Debug)]
pub enum JsonReason {
    Io,
    Syntax,
    Data,
    Eof
}

#[derive(Debug)]
pub enum JURLError {
    DecodingError,
    CompressionError,
    DecompressionError,
    JsonError(JsonReason),
    UTF8Error
}

impl Display for JURLError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            JURLError::CompressionError => f.write_str("CompressionError"),
            JURLError::DecodingError    => f.write_str("DecodingError"),
            JURLError::DecompressionError => f.write_str("DecompressionError"),
            JURLError::JsonError(field) => match field {
                JsonReason::Io => f.write_str("JsonError(JsonReason::Io)"),
                JsonReason::Syntax => f.write_str("JsonError(JsonReason::Syntax)"),
                JsonReason::Data => f.write_str("JsonError(JsonReason::Data)"),
                JsonReason::Eof => f.write_str("JsonError(JsonReason::Eof)"),
            },
            JURLError::UTF8Error => f.write_str("UTF8Error")
        }
    }
}

impl error::Error for JURLError {
    fn description(&self) -> &str {
        match self {
            JURLError::CompressionError => "Could not compress data",
            JURLError::DecodingError => "Could not decode data",
            JURLError::DecompressionError => "Could not decompress data",
            JURLError::JsonError(field) => match field {
                JsonReason::Io => "Failure to read or write bytes on an IO stream.",
                JsonReason::Syntax => "Input is not syntactically valid JSON.",
                JsonReason::Data => "Input data is semantically incorrect.",
                JsonReason::Eof => "Unexcpected end of input data.",
            },
            JURLError::UTF8Error => "Could not decode data as UTF8",
        }    
    }

    fn cause(&self) -> Option<&dyn error::Error> { None }
}

impl From<base64::DecodeError> for JURLError {
    fn from(_e: base64::DecodeError) -> Self {
        JURLError::DecodingError
    }
}

impl From<lzma::error::LzmaError> for JURLError {
    fn from(_e: lzma::error::LzmaError) -> Self {
        JURLError::DecompressionError
    }
}

impl From<std::str::Utf8Error> for JURLError {
    fn from(_e: std::str::Utf8Error) -> Self {
        JURLError::UTF8Error
    }
}

impl From<serde_json::error::Error> for JURLError {
    fn from (e: serde_json::error::Error) -> Self {
        match e.classify() {
            Category::Io => JURLError::JsonError(JsonReason::Io),
            Category::Syntax => JURLError::JsonError(JsonReason::Syntax),
            Category::Data => JURLError::JsonError(JsonReason::Data),
            Category::Eof => JURLError::JsonError(JsonReason::Eof),
        }
    }
}

pub fn pack<T>(content: &T) -> Result<String, JURLError>
where T: serde::Serialize
{
    let json = serde_json::to_string(&content)?;
    let compressed = lzma::compress(json.as_bytes(), 6)?;
    Ok(base64::encode_config(compressed, base64::URL_SAFE_NO_PAD))
}

pub fn unpack<T>(packed: String) -> Result<T, JURLError>
where for <'de> T: serde::Deserialize<'de>
{
    let decoded = base64::decode_config(packed, base64::URL_SAFE_NO_PAD)?;
    let decompressed = lzma::decompress(decoded.as_slice())?;
    let json_string = str::from_utf8(decompressed.as_slice())?;
    match serde_json::from_str::<T>(json_string) {
        Ok(parsed) => Ok(parsed),
        Err(e) => Err(JURLError::from(e))
    }
}