//! Field codec for encoding and decoding fields

use crate::error::{ProtocolError, Result};
use crate::protocol::field::{Field, FieldData, FieldHeader, FieldId};
use bytes::{Buf, BufMut, BytesMut};

/// Decode fields from a buffer
pub fn decode_fields(buf: &mut BytesMut) -> Result<Vec<Field>> {
    if buf.is_empty() {
        return Ok(Vec::new());
    }

    // First 2 bytes: number of fields
    if buf.len() < 2 {
        return Err(ProtocolError::InvalidFieldData);
    }

    let field_count = buf.get_u16() as usize;
    let mut fields = Vec::with_capacity(field_count);

    for _ in 0..field_count {
        if buf.len() < FieldHeader::SIZE {
            return Err(ProtocolError::InvalidFieldData);
        }

        let header = FieldHeader::from_bytes(buf)?;
        buf.advance(FieldHeader::SIZE);

        if buf.len() < header.size as usize {
            return Err(ProtocolError::InvalidFieldData);
        }

        let field_id =
            FieldId::from_u16(header.id).ok_or(ProtocolError::InvalidFieldId(header.id))?;

        // Get field data
        let mut field_data = buf.split_to(header.size as usize);

        // Decode based on common field types
        // Most fields are binary, some are integers
        let data = match field_id {
            FieldId::UserId
            | FieldId::UserIconId
            | FieldId::ChatId
            | FieldId::ChatOptions
            | FieldId::Options
            | FieldId::UserFlags
            | FieldId::Version
            | FieldId::ReferenceNumber
            | FieldId::WaitingCount => {
                // Integer fields (2 or 4 bytes)
                if header.size == 2 {
                    FieldData::Integer(field_data.get_i16() as i32)
                } else if header.size == 4 {
                    FieldData::Integer(field_data.get_i32())
                } else {
                    FieldData::Binary(field_data.to_vec())
                }
            }

            FieldId::UserAccess => {
                // UserAccess is always 8 bytes and needs special bit-reversal handling
                // Store as Binary so it can be decoded with AccessPrivileges::from_wire_format()
                if header.size == 8 {
                    FieldData::Binary(field_data.to_vec())
                } else {
                    // Fallback for incorrect sizes
                    FieldData::Binary(field_data.to_vec())
                }
            }

            FieldId::UserName
            | FieldId::ServerName
            | FieldId::ChatSubject
            | FieldId::FileName
            | FieldId::FileComment => {
                // String fields (try to decode as UTF-8)
                match String::from_utf8(field_data.to_vec()) {
                    Ok(s) => FieldData::String(s),
                    Err(_) => FieldData::Binary(field_data.to_vec()),
                }
            }

            _ => {
                // Default to binary
                FieldData::Binary(field_data.to_vec())
            }
        };

        fields.push(Field { id: field_id, data });
    }

    Ok(fields)
}

/// Encode fields into a buffer
pub fn encode_fields(fields: &[Field], buf: &mut BytesMut) -> Result<()> {
    // Write field count
    buf.put_u16(fields.len() as u16);

    for field in fields {
        // Encode field data first to know the size
        let mut field_buf = BytesMut::new();

        match &field.data {
            FieldData::Integer(v) => {
                // Use appropriate size based on value
                if *v >= i16::MIN as i32 && *v <= i16::MAX as i32 {
                    field_buf.put_i16(*v as i16);
                } else {
                    field_buf.put_i32(*v);
                }
            }
            FieldData::String(s) => {
                field_buf.extend_from_slice(s.as_bytes());
            }
            FieldData::Binary(b) => {
                field_buf.extend_from_slice(b);
            }
        }

        // Write field header
        let header = FieldHeader {
            id: field.id.to_u16(),
            size: field_buf.len() as u16,
        };
        header.to_bytes(buf);

        // Write field data
        buf.extend_from_slice(&field_buf);
    }

    Ok(())
}
