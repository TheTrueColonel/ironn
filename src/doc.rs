use std::cmp::Ordering;
use std::fs;
use std::fs::File;
use std::io::Write;
use color_eyre::Result;
use crate::{FileType};
use crate::app::Position;
use crate::doc_row::Row;

#[derive(Default)]
pub struct Doc {
    rows: Vec<Row>,
    pub file_name: Option<String>,
    pub file_type: FileType,
    dirty: bool,
}

#[allow(clippy::missing_const_for_fn)]
impl Doc {
    pub fn open(filename: &str) -> Result<Self> {
        let contents = fs::read_to_string(filename)?;
        let file_type = FileType::from(filename);
        let mut rows = Vec::new();

        for value in contents.lines() {
            rows.push(Row::from(value));
        }
        
        Ok(
            Self {
                rows,
                file_name: Some(filename.to_owned()),
                file_type,
                dirty: false,
            }
        )
    }
    pub fn insert(&mut self, at: &Position, c: char) {
        if at.y > self.rows.len() {
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
            let row = self.rows.get_mut(at.y).unwrap();
            
            row.insert(at.x, c);
        }
        
        //TODO unhighlight_rows
    }
    pub fn insert_newline(&mut self, at: &Position) {
        match at.y.cmp(&self.rows.len()) {
            Ordering::Greater => return,
            Ordering::Equal => {
                self.rows.push(Row::default());
                return;
            },
            Ordering::Less => ()
        }
        
        let current_row = self.rows.get_mut(at.y).unwrap();
        let new_row = current_row.split(at.x);
        
        self.rows.insert(at.y + 1, new_row);
    }
    pub fn delete(&mut self, at: &Position) {
        if at.y > self.rows.len() {
            return;
        }
        
        self.dirty = true;
        
        // Remove newline and append next line to current line
        if at.x == self.rows.get(at.y).unwrap().len() && at.y + 1 < self.len() {
            let next_row = self.rows.remove(at.y + 1);
            let row = self.rows.get_mut(at.y).unwrap();
            
            row.append(&next_row);
        } else { // Delete like normal
            let row = self.rows.get_mut(at.y).unwrap();
            
            row.delete(at.x);
        }
        
        // TODO unhighlight_rows
    }
    pub fn write_out(&mut self) -> Result<()> {
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
    #[must_use]
    pub fn row(&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows.len()
    }
    #[must_use]
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
}