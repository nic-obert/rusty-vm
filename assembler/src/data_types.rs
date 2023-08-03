use rust_vm_lib::assembly::ByteCode;

use crate::tokenizer::evaluate_string;


/// Represents a data type in static data declarations
pub enum DataType {

    String,
    Char,
    Unsigned1,
    Unsigned2,
    Unsigned4,
    Unsigned8,
    Signed1,
    Signed2,
    Signed4,
    Signed8,
    
}


impl DataType {

    /// Returns a data type from its string name
    pub fn from_name(name: &str) -> Option<Self> {
        match name {

            "string" => Some(DataType::String),
            "char" => Some(DataType::Char),
            "u1" => Some(DataType::Unsigned1),
            "u2" => Some(DataType::Unsigned2),
            "u4" => Some(DataType::Unsigned4),
            "u8" => Some(DataType::Unsigned8),  
            "i1" => Some(DataType::Signed1),
            "i2" => Some(DataType::Signed2),
            "i4" => Some(DataType::Signed4),
            "i8" => Some(DataType::Signed8),

            _ => None,
        }
    }


    /// Encodes a string into a byte code vector based on the data type
    pub fn encode(&self, string: &str, line_number: usize, line: &str) -> ByteCode {

        match self {

            DataType::Char => {
                // Remove the enclosing single quotes
                let string = string.strip_prefix('\'').unwrap_or_else(
                    || crate::error::invalid_data_declaration(line_number, line, "Expected a character literal.")
                ).strip_suffix('\'').unwrap_or_else(
                    || crate::error::invalid_data_declaration(line_number, line, "Expected a character literal.")
                );

                // Evaluate the string by escaping characters
                let evaluated_string = evaluate_string(string, '\'', line_number, line);

                // Check if the character literal is only one character long
                if evaluated_string.len() != 1 {
                    crate::error::invalid_data_declaration(line_number, line, "Character literals must be exactly one character long.");
                }

                // Return the byte representation of the character assuming the string is only one character long
                evaluated_string.into_bytes()
            },

            DataType::String => {
                // Remove the enclosing double quotes
                let string = string.strip_prefix('"').unwrap_or_else(
                    || crate::error::invalid_data_declaration(line_number, line, "Expected a string literal.")
                ).strip_suffix('"').unwrap_or_else(
                    || crate::error::invalid_data_declaration(line_number, line, "Expected a string literal.")
                );

                // Return the evaluated and encoded string
                evaluate_string(string, '"', line_number, line).into_bytes()
            },

            DataType::Unsigned1 => {
                let number = string.parse::<u8>().unwrap_or_else(
                    |_| crate::error::invalid_data_declaration(line_number, line, format!("Expected an unsigned 1 byte integer. Got \"{}\"", string).as_str())
                );
                vec![number]
            },

            DataType::Unsigned2 => {
                let number = string.parse::<u16>().unwrap_or_else(
                    |_| crate::error::invalid_data_declaration(line_number, line, format!("Expected an unsigned 2 byte integer. Got \"{}\"", string).as_str())
                );
                vec![number as u8, (number >> 8) as u8]
            },

            DataType::Unsigned4 => {
                let number = string.parse::<u32>().unwrap_or_else(
                    |_| crate::error::invalid_data_declaration(line_number, line, format!("Expected an unsigned 4 byte integer. Got \"{}\"", string).as_str())
                );
                vec![number as u8, (number >> 8) as u8, (number >> 16) as u8, (number >> 24) as u8]
            },

            DataType::Unsigned8 => {
                let number = string.parse::<u64>().unwrap_or_else(
                    |_| crate::error::invalid_data_declaration(line_number, line, format!("Expected an unsigned 8 byte integer. Got \"{}\"", string).as_str())
                );
                vec![number as u8, (number >> 8) as u8, (number >> 16) as u8, (number >> 24) as u8, (number >> 32) as u8, (number >> 40) as u8, (number >> 48) as u8, (number >> 56) as u8]
            }

            DataType::Signed1 => {
                let number = string.parse::<i8>().unwrap_or_else(
                    |_| crate::error::invalid_data_declaration(line_number, line, format!("Expected a signed 1 byte integer. Got \"{}\"", string).as_str())
                );
                vec![number as u8]
            },

            DataType::Signed2 => {
                let number = string.parse::<i16>().unwrap_or_else(
                    |_| crate::error::invalid_data_declaration(line_number, line, format!("Expected a signed 2 byte integer. Got \"{}\"", string).as_str())
                );
                vec![number as u8, (number >> 8) as u8]
            },

            DataType::Signed4 => {
                let number = string.parse::<i32>().unwrap_or_else(
                    |_| crate::error::invalid_data_declaration(line_number, line, format!("Expected a signed 4 byte integer. Got \"{}\"", string).as_str())
                );
                vec![number as u8, (number >> 8) as u8, (number >> 16) as u8, (number >> 24) as u8]
            },

            DataType::Signed8 => {
                let number = string.parse::<i64>().unwrap_or_else(
                    |_| crate::error::invalid_data_declaration(line_number, line, format!("Expected a signed 8 byte integer. Got \"{}\"", string).as_str())
                );
                vec![number as u8, (number >> 8) as u8, (number >> 16) as u8, (number >> 24) as u8, (number >> 32) as u8, (number >> 40) as u8, (number >> 48) as u8, (number >> 56) as u8]
            }

        }

    }

}


