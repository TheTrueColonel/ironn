use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Row {
    string: String,
    //highlighting,
    len: usize,
    //is_highlighted,
}

#[allow(clippy::missing_const_for_fn)]
impl Row {
    pub fn insert(&mut self, at: usize, c: char) {
        if at >= self.len {
            self.string.push(c);
            self.len += 1;
            
            return;
        }
        
        let mut result = String::new();
        
        for (index, grapheme) in self.string.graphemes(true).enumerate() {
            if index == at {
                result.push(c);
            }
            
            result.push_str(grapheme);
        }
        
        self.len += 1;
        self.string = result;
    }
    pub fn delete(&mut self, at: usize) {
        if at >= self.len {
            return;
        }

        let mut result = String::new();

        for (index, grapheme) in self.string.graphemes(true).enumerate() {
            if index != at {
                result.push_str(grapheme);
            }
        }

        self.len -= 1;
        self.string = result;
    }
    pub fn append(&mut self, new: &Self) {
        self.string = format!("{}{}", self.string, new.string);
        self.len += new.len;
    }
    #[must_use]
    pub fn split(&mut self, at: usize) -> Self {
        let mut row = String::new();
        let mut length = 0;
        let mut split_row = String::new();
        
        for (index, grapheme) in self.string.graphemes(true).enumerate() {
            if index < at {
                length += 1;
                row.push_str(grapheme);
            } else {
                split_row.push_str(grapheme);
            }
        }
        
        let split_length = self.len - length;
        
        self.string = row;
        self.len = length;
        // TODO highlighting
        
        Self {
            string: split_row,
            len: split_length,
        }
    }
    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }
    #[must_use]
    pub fn as_str(&self) -> &str {
        self.string.as_str()
    }
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            //
            len: slice.graphemes(true).count(),
            //
        }
    }
}