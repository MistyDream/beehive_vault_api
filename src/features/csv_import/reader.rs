use std::{iter::Peekable, str::Chars};

pub(super) struct CsvRecord {
    pub line: usize,
    pub fields: Vec<String>,
}

pub(super) struct CsvReader<'a> {
    input: Peekable<Chars<'a>>,
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
        Self {
            input: input
                .strip_prefix('\u{feff}')
                .unwrap_or(input)
                .chars()
                .peekable(),
            delimiter,
            line: 1,
        }
    }

    pub fn next_record(&mut self) -> Result<Option<CsvRecord>, usize> {
        let mut start_line = self.line;
        let mut fields = Vec::new();
        let mut field = String::new();
        let mut state = FieldState::Start;

        while let Some(character) = self.input.next() {
            if character == '\0' {
                return Err(self.line);
            }

            if state == FieldState::Quoted {
                match character {
                    '"' if self.input.next_if_eq(&'"').is_some() => field.push('"'),
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
                    if character == '\r' && self.input.next_if_eq(&'\n').is_none() {
                        return Err(self.line);
                    }
                    self.line += 1;
                    if state == FieldState::Start && fields.is_empty() {
                        start_line = self.line;
                        continue;
                    }

                    fields.push(field);
                    return Ok(Some(CsvRecord {
                        line: start_line,
                        fields,
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
        }))
    }
}
