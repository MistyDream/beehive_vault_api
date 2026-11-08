use std::{iter::Peekable, ops::Range, str::CharIndices};

pub(super) struct CsvRecord {
    pub line: usize,
    pub fields: Vec<String>,
    pub source_range: Range<usize>,
}

pub(super) struct CsvReader<'a> {
    characters: Peekable<CharIndices<'a>>,
    input_len: usize,
    delimiter: char,
    line: usize,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum FieldState {
    Start,
    Unquoted,
    Quoted,
    AfterQuote,
}

impl<'a> CsvReader<'a> {
    pub fn new(input: &'a str, delimiter: char) -> Self {
        let mut characters = input.char_indices().peekable();
        characters.next_if(|(_, character)| *character == '\u{feff}');
        Self {
            characters,
            input_len: input.len(),
            delimiter,
            line: 1,
        }
    }

    pub fn next_record(&mut self) -> Result<Option<CsvRecord>, usize> {
        let mut start_line = self.line;
        let mut start_offset = self.next_offset();
        let mut fields = Vec::new();
        let mut field = String::new();
        let mut state = FieldState::Start;

        while let Some((offset, character)) = self.characters.next() {
            if character == '\0' {
                return Err(self.line);
            }

            if state == FieldState::Quoted {
                match character {
                    '"' if self.characters.next_if(|(_, next)| *next == '"').is_some() => {
                        field.push('"')
                    }
                    '"' => state = FieldState::AfterQuote,
                    '\n' => {
                        self.line += 1;
                        field.push(character);
                    }
                    _ => field.push(character),
                }
                continue;
            }

            match character {
                '\n' | '\r' => {
                    if character == '\r'
                        && self.characters.next_if(|(_, next)| *next == '\n').is_none()
                    {
                        return Err(self.line);
                    }
                    self.line += 1;
                    if state == FieldState::Start && fields.is_empty() {
                        start_line = self.line;
                        start_offset = self.next_offset();
                        continue;
                    }

                    fields.push(field);
                    return Ok(Some(CsvRecord {
                        line: start_line,
                        fields,
                        source_range: start_offset..offset,
                    }));
                }
                character if character == self.delimiter => {
                    fields.push(std::mem::take(&mut field));
                    state = FieldState::Start;
                }
                '"' if state == FieldState::Start => state = FieldState::Quoted,
                '"' => return Err(self.line),
                _ if state == FieldState::AfterQuote => return Err(self.line),
                character => {
                    field.push(character);
                    state = FieldState::Unquoted;
                }
            }
        }

        if state == FieldState::Quoted {
            return Err(start_line);
        }
        if state == FieldState::Start && fields.is_empty() {
            return Ok(None);
        }

        fields.push(field);
        Ok(Some(CsvRecord {
            line: start_line,
            fields,
            source_range: start_offset..self.input_len,
        }))
    }

    fn next_offset(&mut self) -> usize {
        self.characters
            .peek()
            .map_or(self.input_len, |&(offset, _)| offset)
    }
}
