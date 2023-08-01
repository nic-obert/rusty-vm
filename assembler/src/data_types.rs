use rust_vm_lib::assembly::ByteCode;


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
            "s1" => Some(DataType::Signed1),
            "s2" => Some(DataType::Signed2),
            "s4" => Some(DataType::Signed4),
            "s8" => Some(DataType::Signed8),

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

                // Check if the character literal is only one character long
                if string.len() != 1 {
                    crate::error::invalid_data_declaration(line_number, line, "Character literals can only be one character long.");
                }

                // Get the character after the first single quote
                vec![string.chars().next().unwrap() as u8]
            },

            DataType::String => {
                // Remove the enclosing double quotes
                let string = string.strip_prefix('"').unwrap_or_else(
                    || crate::error::invalid_data_declaration(line_number, line, "Expected a string literal.")
                ).strip_suffix('"').unwrap_or_else(
                    || crate::error::invalid_data_declaration(line_number, line, "Expected a string literal.")
                );

                // Handle escape characters and encode the string

                let mut byte_string = Vec::with_capacity(string.len());

                let mut escape_char = false;

                for c in string.chars() {

                    if c == '\\' {
                        if escape_char {
                            byte_string.push('\\' as u8);
                            escape_char = false;
                        } else {
                            escape_char = true;
                            continue;
                        }
                    }

                    if escape_char {
                        match c {
                            'n' => byte_string.push('\n' as u8),
                            't' => byte_string.push('\t' as u8),
                            'r' => byte_string.push('\r' as u8),
                            '0' => byte_string.push('\0' as u8),
                            '"' => byte_string.push('"' as u8),
                            _ => crate::error::invalid_character(c, line_number, 0, line, "Invalid escape character.")
                        }

                        escape_char = false;
                    } else {
                        byte_string.push(c as u8);
                    }

                }

                byte_string
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


