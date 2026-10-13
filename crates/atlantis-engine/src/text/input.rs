use std::io::{self, BufRead};

const C_BUFFER_SIZE: usize = 1023;
const MAX_LINE_BYTES: usize = C_BUFFER_SIZE - 1;

/// Input operations corresponding to `Ainfile` and `Aorders`, over any reader.
pub struct TextInput<R> {
    reader: R,
    failed: bool,
}

impl<R: BufRead> TextInput<R> {
    /// Create a text reader without taking ownership of any filesystem or stdin policy.
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            failed: false,
        }
    }

    /// C++: `fileio.cpp` `Ainfile::GetStr` (lines 168-175).
    pub fn get_str(&mut self) -> io::Result<Option<String>> {
        if self.failed {
            return Ok(None);
        }
        self.skip_white()?;
        self.read_line()
    }

    /// C++: `fileio.cpp` `Ainfile::GetStrNoSkip` (lines 177-183).
    pub fn get_str_no_skip(&mut self) -> io::Result<Option<String>> {
        self.read_line()
    }

    /// C++: `fileio.cpp` `Ainfile::GetInt` (lines 185-189).
    pub fn get_int(&mut self) -> io::Result<Option<i32>> {
        if self.failed {
            return Ok(None);
        }
        self.skip_formatted_whitespace()?;

        let mut token = Vec::new();
        if matches!(self.peek_byte()?, Some(b'+' | b'-')) {
            token.push(self.consume_byte()?.expect("peeked byte exists"));
        }
        while matches!(self.peek_byte()?, Some(b'0'..=b'9')) {
            token.push(self.consume_byte()?.expect("peeked byte exists"));
        }
        if token.is_empty() || token.len() == 1 && matches!(token[0], b'+' | b'-') {
            self.failed = true;
            return Ok(None);
        }

        match std::str::from_utf8(&token)
            .expect("integer tokens are ASCII")
            .parse()
        {
            Ok(value) => Ok(Some(value)),
            Err(_) => {
                self.failed = true;
                Ok(None)
            }
        }
    }

    /// C++: `fileio.cpp` `Aorders::GetLine` (lines 225-233).
    pub fn get_order_line(&mut self) -> io::Result<Option<String>> {
        if self.failed {
            return Ok(None);
        }
        self.skip_white()?;
        self.read_line()
    }

    fn skip_white(&mut self) -> io::Result<()> {
        while matches!(
            self.peek_byte()?,
            Some(b' ' | b'\n' | b'\t' | b'\r' | b'\0')
        ) {
            self.consume_byte()?;
        }
        Ok(())
    }

    fn skip_formatted_whitespace(&mut self) -> io::Result<()> {
        while matches!(self.peek_byte()?, Some(byte) if byte.is_ascii_whitespace()) {
            self.consume_byte()?;
        }
        Ok(())
    }

    fn read_line(&mut self) -> io::Result<Option<String>> {
        if self.failed {
            return Ok(None);
        }

        let mut bytes = Vec::with_capacity(MAX_LINE_BYTES);
        loop {
            if bytes.len() == MAX_LINE_BYTES {
                match self.peek_byte()? {
                    Some(b'\n') => {
                        self.consume_byte()?;
                        break;
                    }
                    Some(_) => self.failed = true,
                    None => {}
                }
                break;
            }

            match self.peek_byte()? {
                Some(b'\n') => {
                    self.consume_byte()?;
                    break;
                }
                Some(_) => bytes.push(self.consume_byte()?.expect("peeked byte exists")),
                None if bytes.is_empty() => return Ok(None),
                None => break,
            }
        }

        if let Some(nul) = bytes.iter().position(|byte| *byte == b'\0') {
            bytes.truncate(nul);
        }
        String::from_utf8(bytes)
            .map(Some)
            .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))
    }

    fn peek_byte(&mut self) -> io::Result<Option<u8>> {
        Ok(self.reader.fill_buf()?.first().copied())
    }

    fn consume_byte(&mut self) -> io::Result<Option<u8>> {
        let byte = self.peek_byte()?;
        if byte.is_some() {
            self.reader.consume(1);
        }
        Ok(byte)
    }
}

#[cfg(test)]
mod tests {
    use super::TextInput;
    use std::io::Cursor;

    // C++: fileio.cpp skipwhite/Ainfile::GetStr, lines 156-175.
    #[test]
    fn get_str_skips_whitespace_and_keeps_trailing_carriage_returns() {
        let mut input = TextInput::new(Cursor::new(&b" \t\r\n\0value\r\nnext"[..]));
        assert_eq!(input.get_str().unwrap().as_deref(), Some("value\r"));
        assert_eq!(input.get_str().unwrap().as_deref(), Some("next"));
        assert_eq!(input.get_str().unwrap(), None);
    }

    // C++: `fileio.cpp` `Ainfile::GetStr` constructs from a NUL-terminated buffer.
    #[test]
    fn get_str_stops_at_an_embedded_nul() {
        let mut input = TextInput::new(Cursor::new(&b"left\0right\n"[..]));
        assert_eq!(input.get_str().unwrap().as_deref(), Some("left"));
    }

    // C++: fileio.cpp Ainfile::GetStrNoSkip, lines 177-183.
    #[test]
    fn get_str_no_skip_returns_empty_lines() {
        let mut input = TextInput::new(Cursor::new(&b"\nnext\n"[..]));
        assert_eq!(input.get_str_no_skip().unwrap().as_deref(), Some(""));
        assert_eq!(input.get_str_no_skip().unwrap().as_deref(), Some("next"));
        assert_eq!(input.get_str_no_skip().unwrap(), None);
    }

    // C++: fileio.cpp Ainfile::GetInt, lines 185-189.
    #[test]
    fn get_int_skips_formatted_whitespace_and_leaves_following_text() {
        let mut input = TextInput::new(Cursor::new(&b" \r\n-42 rest"[..]));
        assert_eq!(input.get_int().unwrap(), Some(-42));
        assert_eq!(input.get_str().unwrap().as_deref(), Some("rest"));
    }

    // C++: `fileio.cpp` `Ainfile::GetInt` leaves the stream failed after bad extraction.
    #[test]
    fn get_int_stops_after_invalid_or_overflowing_numbers() {
        let mut invalid = TextInput::new(Cursor::new(&b"not-a-number 7"[..]));
        assert_eq!(invalid.get_int().unwrap(), None);
        assert_eq!(invalid.get_int().unwrap(), None);

        let mut overflow = TextInput::new(Cursor::new(&b"2147483648 7"[..]));
        assert_eq!(overflow.get_int().unwrap(), None);
        assert_eq!(overflow.get_int().unwrap(), None);
    }

    // C++: fileio.cpp Aorders::GetLine/skipwhite, lines 156-166 and 225-233.
    #[test]
    fn get_order_line_skips_blank_lines_but_does_not_strip_trailing_cr() {
        let mut input = TextInput::new(Cursor::new(&b" \t\r\n\nunit 3\r\nnext\n"[..]));
        assert_eq!(input.get_order_line().unwrap().as_deref(), Some("unit 3\r"));
        assert_eq!(input.get_order_line().unwrap().as_deref(), Some("next"));
        assert_eq!(input.get_order_line().unwrap(), None);
    }

    // C++: fileio.cpp Ainfile::GetStr/Aorders::GetLine, lines 168-183 and 225-233.
    #[test]
    fn line_reads_use_the_fixed_c_buffer_and_stop_after_overflow() {
        let mut exact = vec![b'x'; 1022];
        exact.extend_from_slice(b"\nnext\n");
        let mut exact_input = TextInput::new(Cursor::new(exact));
        assert_eq!(exact_input.get_order_line().unwrap().unwrap().len(), 1022);
        assert_eq!(
            exact_input.get_order_line().unwrap().as_deref(),
            Some("next")
        );

        let mut bytes = vec![b'x'; 1023];
        bytes.extend_from_slice(b"\nnext\n");
        let mut input = TextInput::new(Cursor::new(bytes));
        assert_eq!(input.get_order_line().unwrap().unwrap().len(), 1022);
        assert_eq!(input.get_order_line().unwrap(), None);
    }
}
