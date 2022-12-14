use std::ops;
use std::cmp::{min, max, Ordering};

use crossterm::style::{Stylize, Color};

use crate::util;

#[derive(Debug, Clone, Copy)]
pub enum Mode {
  Terminal,
  Markup,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Attributes {
  pub bold: bool,
  pub invert: bool,
  pub color: Option<Color>,
  pub background: Option<Color>,
}

impl Attributes {
  pub fn merged(&self, with: &Attributes) -> Attributes {
    Attributes{
      bold: self.bold || with.bold,
      invert: self.invert || with.invert,
      color: util::coalesce(self.color, with.color),
      background: util::coalesce(self.background, with.background),
    }
  }
  
  pub fn render(&self, text: &str) -> String {
    self.render_with_mode(text, Mode::Terminal)
  }
  
  fn render_with_mode(&self, text: &str, mode: Mode) -> String {
    match mode {
      Mode::Terminal => self.render_term(text),
      Mode::Markup   => self.render_html(text),
    }
  }
  
  fn render_term(&self, text: &str) -> String {
    let mut styled = text.stylize();
    if self.bold {
      styled = styled.bold();
    }
    if self.invert {
      styled = styled.reverse();
    }
    if let Some(color) = self.color {
      styled = styled.with(color);
    }
    if let Some(background) = self.background {
      styled = styled.on(background);
    }
    styled.to_string()
  }
  
  fn render_html(&self, text: &str) -> String {
    let mut attrd = String::new();
    if self.bold {
      attrd.push_str("<b>");
    }
    if self.invert {
      attrd.push_str("<invert>");
    }
    if let Some(background) = self.background {
      attrd.push_str(&format!("<bg:{:?}>", background));
    }
    if let Some(color) = self.color {
      attrd.push_str(&format!("<fg:{:?}>", color));
    }
    attrd.push_str(text);
    if let Some(color) = self.color {
      attrd.push_str(&format!("</fg:{:?}>", color));
    }
    if let Some(background) = self.background {
      attrd.push_str(&format!("</bg:{:?}>", background));
    }
    if self.invert {
      attrd.push_str("</invert>");
    }
    if self.bold {
      attrd.push_str("</b>");
    }
    attrd
  }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Span {
  range: ops::Range<usize>,
  attrs: Attributes,
}

impl PartialOrd for Span {
  fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
    Some(self.cmp(other))
  }
}

impl Ord for Span {
  fn cmp(&self, other: &Self) -> Ordering {
    match self.range.start.cmp(&other.range.start) {
      Ordering::Equal => self.range.end.cmp(&other.range.end),
      ord             => ord,
    }
  }
}

impl Span {
  pub fn new(range: ops::Range<usize>, attrs: Attributes) -> Span {
    Span{
      range: range,
      attrs: attrs,
    }
  }
}

#[derive(Debug, Clone)]
pub struct Attributed {
  text: String,
  spans: Vec<Span>,
}

impl Attributed {
  pub fn new() -> Attributed {
    Attributed{
      text: String::new(),
      spans: Vec::new(),
    }
  }
  
  pub fn new_with_str(text: &str, spans: Vec<Span>) -> Attributed {
    Attributed{
      text: text.to_string(),
      spans: spans,
    }
  }
  
  pub fn new_with_string(text: String, spans: Vec<Span>) -> Attributed {
    Attributed{
      text: text,
      spans: spans,
    }
  }
  
  pub fn len(&self) -> usize {
    self.text.len()
  }
  
  pub fn text<'a>(&'a self) -> &'a str {
    &self.text
  }
  
  pub fn spans<'a>(&'a self) -> &'a Vec<Span> {
    &self.spans
  }
  
  pub fn spans_mut<'a>(&'a mut self) -> &'a mut Vec<Span> {
    &mut self.spans
  }
  
  pub fn render(&self) -> String {
    self.render_with_mode(Mode::Terminal)
  }
  
  fn render_with_mode(&self, mode: Mode) -> String {
    render_with_mode(&self.text, &self.spans, mode)
  }
}

pub fn merge(a: Vec<Span>, b: Vec<Span>) -> Vec<Span> {
  let mut res: Vec<Span> = Vec::new();
  let mut dup: Vec<Span> = Vec::with_capacity(a.len() + b.len());
  
  dup.extend(a);
  dup.extend(b);
  dup.sort();
  
  loop {
    match dup.len() {
      0 => break,
      1 => {
        res.push(dup[0].clone());
        break;
      },
      _ => {
        if dup[0].range.end > dup[1].range.start {
            if dup[0].range.start < dup[1].range.start {
              res.push(Span{
                range: dup[0].range.start..dup[1].range.start,
                attrs: dup[0].attrs.clone(),
              });
              dup[0] = Span{
                range: dup[1].range.start..dup[0].range.end,
                attrs: dup[0].attrs.clone(),
              };
            }
            let (end, nxt, attrs) = if dup[0].range.end < dup[1].range.end {
              (dup[0].range.end, dup[1].range.end, dup[1].attrs.clone())
            }else{
              (dup[1].range.end, dup[0].range.end, dup[0].attrs.clone())
            };
            dup[0] = Span{
              range: dup[1].range.start..end,
              attrs: dup[0].attrs.merged(&dup[1].attrs),
            };
            if nxt == end {
              dup.remove(1);
            }else{
              dup[1] = Span{
                range: end..nxt,
                attrs: attrs,
              };
            }
        }
        
        while dup.len() > 1 {
          if dup[0].range.end <= dup[1].range.start {
            res.push(dup[0].clone());
            dup.remove(0);
          }else{
            break;
          }
        }
      },
    };
  }
  
  res
}

pub fn render(text: &str, spans: &Vec<Span>) -> String {
  render_with_mode(text, spans, Mode::Terminal)
}

pub fn render_with_offset(text: &str, boff: usize, spans: &Vec<Span>) -> String {
  render_with_options(text, boff, spans, Mode::Terminal)
}

fn render_with_mode(text: &str, spans: &Vec<Span>, mode: Mode) -> String {
  render_with_options(text, 0, spans, mode)
}

fn render_with_options(text: &str, boff: usize, spans: &Vec<Span>, mode: Mode) -> String {
  let mut dup = spans.clone();
  dup.sort();
  
  let len = text.len();
  let mut x = 0;
  let mut attrd = String::new();
  for span in dup {
    if span.range.end < boff { // skip spans that end before the current offset
      continue;
    }
    let start = min(max(boff, span.range.start) - boff, len);
    if start > x { // copy before span starts
      attrd.push_str(&text[x..start]);
    }
    let end = min(span.range.end - boff, len);
    if end > start { // copy attributed range
      attrd.push_str(&span.attrs.render_with_mode(&text[start..end], mode));
    }
    x = end;
  }
  if x < len {
    attrd.push_str(&text[x..]);
  }
  
  attrd
}

#[cfg(test)]
mod tests {
  use super::*;
  
  #[test]
  fn merge_attributes() {
    let a = Attributes{bold:true,  invert: false, color: None, background: None};
    let b = Attributes{bold:false, invert: true,  color: None, background: None};
    let c = Attributes{bold:false, invert: false, color: Some(Color::Blue), background: None};
    
    assert_eq!(Attributes{bold:true,  invert: true, color: None, background: None}, a.merged(&b));
    assert_eq!(Attributes{bold:false, invert: true, color: Some(Color::Blue), background: None}, c.merged(&b));
  }
  
  #[test]
  fn merge_spans() {
    let a = vec![
      Span::new(0..5, Attributes{bold:true,  invert: false, color: None, background: None}),
    ];
    let b = vec![
      Span::new(0..5, Attributes{bold:false, invert: false, color: Some(Color::Blue), background: None}),
    ];
    assert_eq!(vec![
      Span::new(0..5, Attributes{bold:true, invert: false, color: Some(Color::Blue), background: None}),
    ], merge(a, b));
    
    let a = vec![
      Span::new(0..5, Attributes{bold:true,  invert: false, color: None, background: None}),
    ];
    let b = vec![
      Span::new(3..5, Attributes{bold:false, invert: false, color: Some(Color::Blue), background: None}),
    ];
    assert_eq!(vec![
      Span::new(0..3, Attributes{bold:true, invert: false, color: None, background: None}),
      Span::new(3..5, Attributes{bold:true, invert: false, color: Some(Color::Blue), background: None}),
    ], merge(a, b));
    
    let a = vec![
      Span::new(3..5, Attributes{bold:false, invert: false, color: Some(Color::Blue), background: None}),
      Span::new(3..5, Attributes{bold:true,  invert: true,  color: None, background: None}),
      Span::new(0..5, Attributes{bold:false, invert: false, color: None, background: None}),
    ];
    let b = vec![
      Span::new(3..5, Attributes{bold:false, invert: false, color: Some(Color::Blue), background: None}),
      Span::new(3..5, Attributes{bold:true,  invert: true,  color: None, background: None}),
    ];
    assert_eq!(vec![
      Span::new(0..3, Attributes{bold:false, invert: false, color: None, background: None}),
      Span::new(3..5, Attributes{bold:true,  invert: true,  color: Some(Color::Blue), background: None}),
    ], merge(a, b));
    
    let a = vec![
      Span::new(3..5, Attributes{bold:false, invert: false, color: Some(Color::Red), background: None}), // first non-null color prevails
      Span::new(0..5, Attributes{bold:false, invert: false, color: None, background: None}),
    ];
    let b = vec![
      Span::new(3..5, Attributes{bold:true,  invert: true,  color: None, background: None}),
      Span::new(3..5, Attributes{bold:false, invert: false, color: Some(Color::Blue), background: None}),
    ];
    assert_eq!(vec![
      Span::new(0..3, Attributes{bold:false, invert: false, color: None, background: None}),
      Span::new(3..5, Attributes{bold:true,  invert: true,  color: Some(Color::Red), background: None}),
    ], merge(a, b));
  }
  
  #[test]
  fn render_attributes() {
    let t = "Hello, there.";
    
    let a = vec![Span::new(0..5, Attributes{bold:true, invert: false, color: None, background: None})];
    assert_eq!("<b>Hello</b>, there.", render_with_mode(t, &a, Mode::Markup));
    
    let a = vec![Span::new(0..5, Attributes{bold:true, invert: false, color: Some(Color::Blue), background: None})];
    assert_eq!("<b><fg:Blue>Hello</fg:Blue></b>, there.", render_with_mode(t, &a, Mode::Markup));
    
    let a = vec![Span::new(7..12, Attributes{bold:false, invert: false, color: Some(Color::Green), background: None}), Span::new(0..5, Attributes{bold:true, invert: false, color: Some(Color::Blue), background: None})];
    assert_eq!("<b><fg:Blue>Hello</fg:Blue></b>, <fg:Green>there</fg:Green>.", render_with_mode(t, &a, Mode::Markup));
  }
  
  #[test]
  fn render_attributes_with_offset() {
    let t = "Hello, there.";
    let x = 7;
    let p = &t[x..];
    
    let a = vec![Span::new(7..12, Attributes{bold:false, invert: false, color: Some(Color::Green), background: None}), Span::new(12..13, Attributes{bold:true, invert: false, color: None, background: None})];
    assert_eq!("<fg:Green>there</fg:Green><b>.</b>", render_with_options(p, x, &a, Mode::Markup));
  }
  
  #[test]
  fn render_attributed() {
    let t = "Hello, there.";
    
    let a = Attributed::new_with_str(t, vec![Span::new(0..5, Attributes{bold:true, invert: false, color: None, background: None})]);
    assert_eq!("<b>Hello</b>, there.", a.render_with_mode(Mode::Markup));
    
    let a = Attributed::new_with_str(t, vec![Span::new(0..5, Attributes{bold:true, invert: false, color: None, background: None})]);
    assert_eq!("<b>Hello</b>, there.", a.render_with_mode(Mode::Markup));
    
    let a = Attributed::new_with_str(t, vec![Span::new(0..5, Attributes{bold:true, invert: false, color: Some(Color::Blue), background: None})]);
    assert_eq!("<b><fg:Blue>Hello</fg:Blue></b>, there.", a.render_with_mode(Mode::Markup));
    
    let a = Attributed::new_with_str(t, vec![
      Span::new(7..12, Attributes{bold:false, invert: false, color: Some(Color::Green), background: None}), // deliberately out of order
      Span::new(0..5, Attributes{bold:true, invert: false, color: Some(Color::Blue), background: None})
    ]);
    assert_eq!("<b><fg:Blue>Hello</fg:Blue></b>, <fg:Green>there</fg:Green>.", a.render_with_mode(Mode::Markup));
  }
  
}
