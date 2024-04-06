use std::fs;
use std::fs::File;
use std::io::Write;
use color_eyre::Result;
use crate::{FileType, Position, Row, SearchDirection};

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    dirty: bool,
    file_type: FileType,
}

impl Document {
    pub fn open(filename: &str) -> Result<Self> {
        let contents = fs::read_to_string(filename)?;
        let file_type = FileType::from(filename);
        let mut rows = Vec::new();
        
        for value in contents.lines() {
            rows.push(Row::from(value));
        }
        
        Ok(Self {
            rows,
            file_name: Some(filename.to_owned()),
            dirty: false,
            file_type,
        })
    }
    #[must_use]
    pub fn file_type(&self) -> String {
        self.file_type.name()
    }
    #[must_use]
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    #[must_use]
    #[allow(clippy::missing_const_for_fn)]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    // If user is typing on last line, add new row, otherwise type as normal
    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y - 1 > self.rows.len() {
            return;
        }

        self.dirty = true;

        if c == '\n' {
            self.insert_newline(at);
        } else if at.y == self.rows.len() {
            let mut row = Row::default();
            row.insert(0, c);
            self.rows.push(row);
        } else {
            let row = &mut self.rows[at.y - 1];
            row.insert(at.x, c);
        }
        
        self.unhighlight_rows(at.y);
    }
    pub fn insert_newline(&mut self, at: &Position) {
        if at.y > self.rows.len() {
            return;
        }
        if at.y == self.rows.len() {
            self.rows.push(Row::default());
            return;
        }

        let current_row = &mut self.rows[at.y];
        let new_row = current_row.split(at.x);

        self.rows.insert(at.y + 1, new_row);
    }
    pub fn highlight(&mut self, word: &Option<String>, until: Option<usize>) {
        let mut  start_with_comment = false;
        
        let until = if let Some(until) = until {
            if until.saturating_add(1) < self.rows.len() {
                until.saturating_add(1)
            } else {
                self.rows.len()
            }
        } else {
            self.rows.len()
        };

        for row in &mut self.rows[..until] {
            start_with_comment = row.highlight(self.file_type.highlighting_options(), word, start_with_comment);
        }
    }
    pub fn unhighlight_rows(&mut self, start: usize) {
        let start = start.saturating_sub(1);
        
        for row in self.rows.iter_mut().skip(start) {
            row.unhighlight();
        }
    }
    pub fn delete(&mut self, at: &Position) {
        let len = self.len();
        
        if at.y >= len {
            return;
        }

        self.dirty = true;

        // Remove newline and append next line to current line
        if at.x == self.rows[at.y].len() && at.y + 1 < len {
            let next_row = self.rows.remove(at.y + 1);
            let row = &mut self.rows[at.y];
            
            row.append(&next_row);
        } else { // Delete like normal
            let row = &mut self.rows[at.y];
            
            row.delete(at.x);
        }
        
        self.unhighlight_rows(at.y);
    }
    pub fn save(&mut self) -> Result<()> {
        if let Some(file_name) = &self.file_name {
            let mut file = File::create(file_name)?;
            
            self.file_type = FileType::from(file_name);
            
            for row in &mut self.rows {
                file.write_all(row.as_bytes())?;
                file.write_all(b"\n")?;
            }

            self.dirty = false;
        }
        
        Ok(())
    }
    // Iterate over all rows and call their find methods returning the row (y) and column (x) of a found query
    // Return `None` if not found
    #[must_use]
    pub fn find(&self, query: &str, at: &Position, direction: SearchDirection) -> Option<Position> {
        if at.y >= self.rows.len() {
            return None;
        }
        
        let mut position = Position { x: at.x, y: at.y };
        let start = if direction == SearchDirection::Forward {
            at.y
        } else {
            0
        };
        let end = if direction == SearchDirection::Forward {
            self.rows.len()
        } else {
            at.y.saturating_add(1)
        };
        
        for _ in start..end {
            if let Some(row) = self.rows.get(position.y) {
                if let Some(x) = row.find(query, position.x, direction) {
                    position.x = x;
                    return Some(position);
                }
                
                if direction == SearchDirection::Forward {
                    position.y = position.y.saturating_add(1);
                    position.x = 0;
                } else {
                    position.y = position.y.saturating_sub(1);
                    position.x = self.rows[position.y].len();
                }
            } else {
                return None;
            }
        }
        
        None
    }
}