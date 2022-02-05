use crate::Row;
use std::fs;

#[derive(Default)]
pub struct Document {
    rows: Vec<Row>,
}

impl Document {
    pub fn open (filename: &str) -> Result<Self, std::io::Error> {
        let contents = fs::read_to_string(filename)?;
        let mut rows = Vec::new();
        for val in contents.lines() {
            rows.push(Row::from(val));
        }
        Ok(Self { rows })
    }

    pub fn row (&self, index: usize) -> Option<&Row> {
        self.rows.get(index)
    }

    pub fn is_empty (&self) -> bool {
        self.rows.is_empty()
    }
}
