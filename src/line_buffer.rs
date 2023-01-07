use unicode_segmentation::UnicodeSegmentation;

pub struct LineBuffer {
    buffer: String,
    insertion_point: usize,
}

impl LineBuffer {
    pub fn new() -> Self {
        Self {
            buffer: String::new(),
            insertion_point: 0,
        }
    }

    fn get_grapheme_indices(&self) -> Vec<(usize, &str)> {
        UnicodeSegmentation::grapheme_indices(self.buffer.as_str(), true).collect()
    }

    pub fn set_insertion_point(&mut self, pos: usize) {
        self.insertion_point = pos;
    }

    pub fn get_insertion_point(&self) -> usize {
        self.insertion_point
    }

    pub fn get_buffer_length(&self) -> usize {
        self.buffer.len()
    }

    pub fn get_buffer(&self) -> &str {
        &self.buffer
    }

    pub fn slice_buffer(&self, pos: usize) -> &str {
        &self.buffer[pos..]
    }

    pub fn increment_insertion_point(&mut self) {
        let grapheme_indices = self.get_grapheme_indices();
        for i in 0..grapheme_indices.len() {
            if grapheme_indices[i].0 == self.insertion_point && i < (grapheme_indices.len() - 1) {
                self.insertion_point = grapheme_indices[i + 1].0;
                return;
            }
        }
        self.insertion_point = self.buffer.len();
    }

    pub fn decrement_insertion_point(&mut self) {
        let grapheme_indices = self.get_grapheme_indices();
        if self.insertion_point == self.get_buffer_length() {
            self.insertion_point = match grapheme_indices.last() {
                Some((i, _)) => *i,
                None => 0,
            }
        } else {
            for i in 0..grapheme_indices.len() {
                if grapheme_indices[i].0 == self.insertion_point && i > 1 {
                    self.insertion_point = grapheme_indices[i - 1].0;
                    return;
                }
            }
            self.insertion_point = 0;
        }
    }

    pub fn insert_char(&mut self, pos: usize, c: char) {
        self.buffer.insert(pos, c);
    }

    pub fn remove_char(&mut self, pos: usize) {
        self.buffer.remove(pos);
    }

    pub fn is_empty(&self) -> bool {
        self.buffer.is_empty()
    }

    pub fn pop(&mut self) -> Option<char> {
        self.buffer.pop()
    }

    pub fn move_word_left(&mut self) -> usize {
        match self
            .buffer
            .rmatch_indices(&[' ', '\t'][..])
            .find(|(index, _)| index < &(self.insertion_point - 1))
        {
            Some((index, _)) => {
                self.insertion_point = index;
            }
            None => {
                self.insertion_point = 0;
            }
        }
        self.insertion_point
    }

    pub fn move_word_right(&mut self) -> usize {
        match self
            .buffer
            .match_indices(&[' ', '\t'][..])
            .find(|(index, _)| index > &self.insertion_point)
        {
            Some((index, _)) => {
                self.insertion_point = index + 1;
            }
            None => {
                self.insertion_point = self.get_buffer_length();
            }
        }
        self.insertion_point
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }
}

#[test]
fn emoji_test() {
    // "🤣"
    // "🙇♀️"
}
