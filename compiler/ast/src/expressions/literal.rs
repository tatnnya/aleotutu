// Copyright (C) 2019-2022 Aleo Systems Inc.
// This file is part of the Leo library.

// The Leo library is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// The Leo library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with the Leo library. If not, see <https://www.gnu.org/licenses/>.

use crate::GroupLiteral;

use super::*;

/// A literal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Literal {
    // todo: deserialize values here
    /// An address literal, e.g., `aleo1qnr4dkkvkgfqph0vzc3y6z2eu975wnpz2925ntjccd5cfqxtyu8s7pyjh9`.
    Address(String, #[serde(with = "leo_span::span_json")] Span),
    /// A boolean literal, either `true` or `false`.
    Boolean(bool, #[serde(with = "leo_span::span_json")] Span),
    /// A field literal, e.g., `42field`.
    /// A signed number followed by the keyword `field`.
    Field(String, #[serde(with = "leo_span::span_json")] Span),
    /// A group literal, either product or affine.
    /// For example, `42group` or `(12, 52)group`.
    Group(Box<GroupLiteral>),
    /// An 8-bit signed integer literal, e.g., `42i8`.
    I8(String, #[serde(with = "leo_span::span_json")] Span),
    /// A 16-bit signed integer literal, e.g., `42i16`.
    I16(String, #[serde(with = "leo_span::span_json")] Span),
    /// A 32-bit signed integer literal, e.g., `42i32`.
    I32(String, #[serde(with = "leo_span::span_json")] Span),
    /// A 64-bit signed integer literal, e.g., `42i64`.
    I64(String, #[serde(with = "leo_span::span_json")] Span),
    /// A 128-bit signed integer literal, e.g., `42i128`.
    I128(String, #[serde(with = "leo_span::span_json")] Span),
    /// A scalar literal, e.g. `1scalar`.
    /// An unsigned number followed by the keyword `scalar`.
    Scalar(String, #[serde(with = "leo_span::span_json")] Span),
    /// A string literal, e.g., `"foobar"`.
    String(String, #[serde(with = "leo_span::span_json")] Span),
    /// A 8-bit unsigned integer literal, e.g., `42u8`.
    U8(String, #[serde(with = "leo_span::span_json")] Span),
    /// A 16-bit unsigned integer literal, e.g., `42u16`.
    U16(String, #[serde(with = "leo_span::span_json")] Span),
    /// A 32-bit unsigned integer literal, e.g., `42u32`.
    U32(String, #[serde(with = "leo_span::span_json")] Span),
    /// A 64-bit unsigned integer literal, e.g., `42u64`.
    U64(String, #[serde(with = "leo_span::span_json")] Span),
    /// A 128-bit unsigned integer literal, e.g., `42u128`.
    U128(String, #[serde(with = "leo_span::span_json")] Span),
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self {
            Self::Address(address, _) => write!(f, "{}", address),
            Self::Boolean(boolean, _) => write!(f, "{}", boolean),
            Self::Field(field, _) => write!(f, "{}", field),
            Self::Group(group) => write!(f, "{}", group),
            Self::I8(integer, _) => write!(f, "{}", integer),
            Self::I16(integer, _) => write!(f, "{}", integer),
            Self::I32(integer, _) => write!(f, "{}", integer),
            Self::I64(integer, _) => write!(f, "{}", integer),
            Self::I128(integer, _) => write!(f, "{}", integer),
            Self::Scalar(scalar, _) => write!(f, "{}", scalar),
            Self::String(string, _) => write!(f, "{}", string),
            Self::U8(integer, _) => write!(f, "{}", integer),
            Self::U16(integer, _) => write!(f, "{}", integer),
            Self::U32(integer, _) => write!(f, "{}", integer),
            Self::U64(integer, _) => write!(f, "{}", integer),
            Self::U128(integer, _) => write!(f, "{}", integer),
        }
    }
}

impl Node for Literal {
    fn span(&self) -> Span {
        match &self {
            Self::Address(_, span)
            | Self::Boolean(_, span)
            | Self::Field(_, span)
            | Self::I8(_, span)
            | Self::I16(_, span)
            | Self::I32(_, span)
            | Self::I64(_, span)
            | Self::I128(_, span)
            | Self::Scalar(_, span)
            | Self::String(_, span)
            | Self::U8(_, span)
            | Self::U16(_, span)
            | Self::U32(_, span)
            | Self::U64(_, span)
            | Self::U128(_, span) => *span,
            Self::Group(group) => match &**group {
                GroupLiteral::Single(_, span) => *span,
                GroupLiteral::Tuple(tuple) => tuple.span,
            },
        }
    }

    fn set_span(&mut self, new_span: Span) {
        match self {
            Self::Address(_, span)
            | Self::Boolean(_, span)
            | Self::Field(_, span)
            | Self::I8(_, span)
            | Self::I16(_, span)
            | Self::I32(_, span)
            | Self::I64(_, span)
            | Self::I128(_, span)
            | Self::Scalar(_, span)
            | Self::String(_, span)
            | Self::U8(_, span)
            | Self::U16(_, span)
            | Self::U32(_, span)
            | Self::U64(_, span)
            | Self::U128(_, span) => *span = new_span,
            Self::Group(group) => match &mut **group {
                GroupLiteral::Single(_, span) => *span = new_span,
                GroupLiteral::Tuple(tuple) => tuple.span = new_span,
            },
        }
    }
}
