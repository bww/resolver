use std::io;
use std::io::stdout;
use std::str;

use crate::rows::Rows;

pub struct Buffer {
  data: String,
}

impl Buffer {
  pub fn new() -> Self {
    Buffer{
      data: String::new(),
    }
  }
  
  pub fn text(&self) -> &str {
    &self.data
  }
  
  pub fn lines(&self) -> str::Lines {
    self.data.lines()
  }
  
  pub fn push(&mut self, c: char) {
    self.data.push(c);
  }
  
  pub fn push_str(&mut self, s: &str) {
    self.data.push_str(s);
  }
  
  pub fn push_rows(&mut self, rows: &Rows) {
    self.data.push_str(&rows.render());
  }
  
  pub fn clear(&mut self) {
    self.data.clear();
  }
}

impl io::Write for Buffer {
  fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
    match std::str::from_utf8(buf) {
      Ok(v) => {
        self.push_str(v);
        Ok(v.len())
      },
      Err(_) => Err(io::ErrorKind::WriteZero.into()),
    }
  }
  
  fn flush(&mut self) -> io::Result<()> {
    let out = write!(stdout(), "{}", self.data);
    stdout().flush()?;
    self.data.clear();
    out
  }
}