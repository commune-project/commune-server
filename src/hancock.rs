// Taken from https://git.sr.ht/~vpzom/hancock/tree/master/item/src/lib.rs

// The MIT License (MIT)

// Copyright (c) 2020 Colin Reeder
// Copyright (c) 2021 Misaka 0x4e21

// Permission is hereby granted, free of charge, to any person obtaining a copy
// of this software and associated documentation files (the "Software"), to deal
// in the Software without restriction, including without limitation the rights
// to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
// copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:

// The above copyright notice and this permission notice shall be included in all
// copies or substantial portions of the Software.

// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
// IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
// FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
// AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
// LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
// SOFTWARE.

//! HTTP Signature handling utility.
//!
//! More details in the `Signature` struct
#![warn(missing_docs)]

use warp::http;
use thiserror;

/// Errors that may be produced when parsing a signature header
#[derive(Debug, thiserror::Error)]
pub enum ParseError {
    /// A parameter pair did not contain `=`
    #[error("Parameter pair did not contain =")]
    MissingEquals,

    /// Didn't find a signature in the header
    #[error("No signature found in parameters")]
    MissingSignature,

    /// A parameter contained invalid characters
    #[error("Parameter contained invalid characters")]
    InvalidCharacters,

    /// Failed to parse a number
    #[error("Failed to parse number")]
    Number(std::num::ParseIntError),

    /// Signature field was not valid Base64
    #[error("Failed to parse signature bytes")]
    Base64(base64::DecodeError),
}

/// Errors that may be produced when creating a signature
#[derive(Debug, thiserror::Error)]
pub enum SignError<T: std::fmt::Debug> {
    /// An IO error occurred.
    #[error("IO error occurred")]
    IO(#[from] std::io::Error),

    /// An error was returned from the provided `sign` function.
    #[error("Failed in user sign call")]
    User(T),
}

/// Errors that may be produced when verifying a signature
#[derive(Debug, thiserror::Error)]
pub enum VerifyError<T: std::fmt::Debug> {
    /// An IO error occurred.
    #[error("IO error occurred")]
    IO(#[from] std::io::Error),

    /// An error was returned from the provided `verify` function.
    #[error("Failed in user verify call")]
    User(T),
}

/// Header name enum of HTTP Signatures, supporting (request-target) etc.
#[derive(Clone)]
pub enum SignatureHeaderName {
    /// special header (request-target) of HTTP Signatures spec.
    RequestTarget,
    /// special header (created) of HTTP Signatures spec.
    Created,
    /// special header (expires) of HTTP Signatures spec.
    Expires,
    /// Normal header
    NormalHeader(http::header::HeaderName),
}

impl SignatureHeaderName {
    /// Serialize to &str.
    pub fn as_str(&self) -> &str {
        match self {
            SignatureHeaderName::RequestTarget => "(request-target)",
            SignatureHeaderName::Created => "(created)",
            SignatureHeaderName::Expires => "(expires)",
            SignatureHeaderName::NormalHeader(header) => header.as_str(),
        }
    }
}

impl From<http::header::HeaderName> for SignatureHeaderName {
    fn from(src: http::header::HeaderName) -> Self {
        SignatureHeaderName::NormalHeader(src)
    }
}

impl std::str::FromStr for SignatureHeaderName {
    type Err = http::header::InvalidHeaderName;

    fn from_str(src: &str) -> Result<Self, http::header::InvalidHeaderName> {
        if src == "(request-target)" {
            Ok(SignatureHeaderName::RequestTarget)
        } else if src == "(created)" {
            Ok(SignatureHeaderName::Created)
        } else if src == "(expires)" {
            Ok(SignatureHeaderName::Expires)
        } else {
            Ok(SignatureHeaderName::NormalHeader(src.parse()?))
        }
    }
}

fn parse_maybe_quoted<'a>(src: &'a str) -> &'a str {
    // TODO handle escapes?

    if src.starts_with('"') && src.ends_with('"') {
        &src[1..(src.len() - 1)]
    } else {
        src
    }
}

/// A parsed or generated Signature.
#[derive(Clone)]
pub struct Signature {
    /// Signature hash algorithm.
    pub algorithm: Option<http::header::HeaderName>,
    /// (created) header.
    pub created: Option<u64>,
    /// (expires) header.
    pub expires: Option<u64>,
    /// (signed headers) header.
    pub headers: Option<Vec<SignatureHeaderName>>,
    /// keyId of HTTP Signatures.
    pub key_id: Option<String>,
    /// signature data.
    pub signature: Vec<u8>,
}

impl Signature {
    /// Construct a signature.
    ///
    /// All headers in `headers` will be included, as well as `(request-target)`, `(created)`, and
    /// `(expires)` (based on `lifetime_secs` parameter)
    ///
    /// The passed `sign` will be called with the body to sign.
    pub fn create<E: std::fmt::Debug>(
        key_id: &str,
        request_method: &http::method::Method,
        request_path_and_query: &str,
        lifetime_secs: u64,
        headers: &http::header::HeaderMap,
        sign: impl FnOnce(Vec<u8>) -> Result<Vec<u8>, E>,
    ) -> Result<Self, SignError<E>> {
        use std::io::Write;

        let created = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Timestamp is wildly unrealistic (before epoch)")
            .as_secs();
        let expires = created + lifetime_secs;

        let mut body = Vec::new();

        write!(
            body,
            "(request-target): {} {}\n(created): {}\n(expires): {}",
            request_method.as_str().to_lowercase(),
            request_path_and_query,
            created,
            expires,
        )?;

        for name in headers.keys() {
            write!(body, "\n{}: ", name)?;

            let mut first = true;
            for value in headers.get_all(name) {
                if first {
                    first = false;
                } else {
                    write!(body, ", ")?;
                }

                body.extend(value.as_bytes());
            }
        }

        let header_names: Vec<_> = vec![
            SignatureHeaderName::RequestTarget,
            SignatureHeaderName::Created,
            SignatureHeaderName::Expires,
        ]
        .into_iter()
        .chain(headers.keys().cloned().map(Into::into))
        .collect();

        let signature = sign(body).map_err(SignError::User)?;

        Ok(Self {
            algorithm: Some(http::header::HeaderName::from_static("hs2019")),
            created: Some(created),
            expires: Some(expires),
            headers: Some(header_names),
            key_id: Some(String::from(key_id)),
            signature,
        })
    }

    /// Create an old-style signature (no (created) and (expires))
    ///
    /// # Panics
    /// Panics if `headers` doesn't contain a Date header
    pub fn create_legacy<E: std::fmt::Debug>(
        key_id: &str,
        request_method: &http::method::Method,
        request_path_and_query: &str,
        headers: &http::header::HeaderMap,
        sign: impl FnOnce(Vec<u8>) -> Result<Vec<u8>, E>,
    ) -> Result<Self, SignError<E>> {
        use std::io::Write;

        if !headers.contains_key(http::header::DATE) {
            panic!("legacy signatures must contain Date header");
        }

        let mut body = Vec::new();

        write!(
            body,
            "(request-target): {} {}",
            request_method.as_str().to_lowercase(),
            request_path_and_query,
        )?;

        for name in headers.keys() {
            write!(body, "\n{}: ", name)?;

            let mut first = true;
            for value in headers.get_all(name) {
                if first {
                    first = false;
                } else {
                    write!(body, ", ")?;
                }

                body.extend(value.as_bytes());
            }
        }

        let header_names: Vec<_> = std::iter::once(SignatureHeaderName::RequestTarget)
            .chain(headers.keys().cloned().map(Into::into))
            .collect();

        let signature = sign(body).map_err(SignError::User)?;

        Ok(Self {
            algorithm: Some(http::header::HeaderName::from_static("hs2019")),
            created: None,
            expires: None,
            headers: Some(header_names),
            key_id: Some(String::from(key_id)),
            signature,
        })
    }

    /// Parse a Signature header
    pub fn parse(value: &http::header::HeaderValue) -> Result<Self, ParseError> {
        let mut algorithm = None;
        let mut created = None;
        let mut expires = None;
        let mut headers = None;
        let mut key_id = None;
        let mut signature = None;

        for field_src in value
            .to_str()
            .map_err(|_| ParseError::InvalidCharacters)?
            .split(',')
        {
            let eqidx = field_src.find('=').ok_or(ParseError::MissingEquals)?;

            let key = &field_src[..eqidx];
            let value = parse_maybe_quoted(&field_src[(eqidx + 1)..]);

            match key {
                "algorithm" => {
                    algorithm = Some(value.parse().map_err(|_| ParseError::InvalidCharacters)?);
                }
                "created" => {
                    created = Some(value.parse().map_err(ParseError::Number)?);
                }
                "expires" => {
                    expires = Some(value.parse().map_err(ParseError::Number)?);
                }
                "headers" => {
                    headers = Some(
                        value
                            .split(' ')
                            .map(|x| x.parse().map_err(|_| ParseError::InvalidCharacters))
                            .collect::<Result<Vec<_>, _>>()?,
                    );
                }
                "keyId" => {
                    key_id = Some(String::from(value));
                }
                "signature" => {
                    signature = Some(base64::decode(value).map_err(ParseError::Base64)?);
                }
                _ => {}
            }
        }

        Ok(Self {
            algorithm,
            created,
            expires,
            headers,
            key_id,
            signature: signature.ok_or(ParseError::MissingSignature)?,
        })
    }

    /// Create a Signature header value for the signature.
    pub fn to_header(&self) -> http::header::HeaderValue {
        use std::fmt::Write;
        let mut params = String::new();

        write!(params, "headers=\"").unwrap();
        if let Some(ref headers) = self.headers {
            for (idx, name) in headers.iter().enumerate() {
                if idx != 0 {
                    write!(params, " ").unwrap();
                }
                write!(params, "{}", name.as_str()).unwrap();
            }
        } else {
            write!(params, "(created)").unwrap();
        }
        write!(params, "\"").unwrap();

        if let Some(ref algorithm) = self.algorithm {
            write!(params, ",algorithm={}", algorithm).unwrap();
        }
        if let Some(created) = self.created {
            write!(params, ",created={}", created).unwrap();
        }
        if let Some(expires) = self.expires {
            write!(params, ",expires={}", expires).unwrap();
        }
        if let Some(key_id) = self.key_id.as_ref().map(|s| s.as_str()) {
            write!(params, ",keyId=\"{}\"", key_id).unwrap();
        }

        write!(params, ",signature=\"").unwrap();
        base64::encode_config_buf(&self.signature, base64::STANDARD, &mut params);
        write!(params, "\"").unwrap();

        http::header::HeaderValue::from_bytes(params.as_bytes()).unwrap()
    }

    /// Verify the signature for a given request target and HeaderMap.
    ///
    /// The passed `verify` function will be called with (body, signature) where body is the body
    /// that should match the signature.
    pub fn verify<E: std::fmt::Debug>(
        &self,
        request_method: &http::method::Method,
        request_path_and_query: &str,
        headers: &http::header::HeaderMap,
        verify: impl FnOnce(&[u8], &[u8]) -> Result<bool, E>,
    ) -> Result<bool, VerifyError<E>> {
        use std::io::Write;

        let now = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .expect("Timestamp is wildly inaccurate")
            .as_secs();

        if let Some(expires) = self.expires {
            if expires < now {
                return Ok(false);
            }
        }

        let mut body = Vec::new();
        if let Some(header_names) = &self.headers {
            for (idx, name) in header_names.iter().enumerate() {
                if idx != 0 {
                    write!(body, "\n")?;
                }
                match name {
                    SignatureHeaderName::RequestTarget => {
                        write!(
                            body,
                            "(request-target): {} {}",
                            request_method.as_str().to_lowercase(),
                            request_path_and_query
                        )?;
                    }
                    SignatureHeaderName::Created => {
                        if let Some(created) = self.created {
                            write!(body, "(created): {}", created)?;
                        } else {
                            return Ok(false);
                        }
                    }
                    SignatureHeaderName::Expires => {
                        if let Some(expires) = self.expires {
                            write!(body, "(expires): {}", expires)?;
                        } else {
                            return Ok(false);
                        }
                    }
                    SignatureHeaderName::NormalHeader(name) => {
                        write!(body, "{}: ", name)?;

                        let mut first = true;
                        for value in headers.get_all(name) {
                            if first {
                                first = false;
                            } else {
                                write!(body, ", ")?;
                            }

                            body.extend(value.as_bytes());
                        }
                    }
                }
            }
        } else {
            if let Some(created) = self.created {
                write!(body, "(created): {}", created)?;
            } else {
                return Ok(false);
            }
        }

        verify(&body, &self.signature).map_err(VerifyError::User)
    }
}
