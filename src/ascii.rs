use std::fmt::Display;

pub const RESET: &str = "\x1b[0m";
pub const BOLD: &str = "\x1b[1m";
pub const UNDERLINE: &str = "\x1b[4m";

pub trait Color {
  fn rgb(&self, r: u8, g: u8, b: u8) -> String
  where
    Self: Display,
  {
    format!("\x1b[38;2;{r};{g};{b}m{self}")
  }
  fn on_rgb(&self, r: u8, g: u8, b: u8) -> String
  where
    Self: Display,
  {
    format!("\x1b[48;2;{r};{g};{b}m{self}")
  }
  fn bold(&self) -> String
  where
    Self: Display,
  {
    format!("{BOLD}{self}")
  }
  fn underline(&self) -> String
  where
    Self: Display,
  {
    format!("{UNDERLINE}{self}")
  }
  fn reset(&self) -> String
  where
    Self: Display,
  {
    format!("{self}{RESET}")
  }
  fn err(&self) -> String
  where
    Self: Display,
  {
    self.bold().underline().rgb(255, 75, 75).reset()
  }
  fn success(&self) -> String
  where
    Self: Display,
  {
    self.bold().underline().rgb(0, 255, 94).reset()
  }
  fn info(&self) -> String
  where
    Self: Display,
  {
    self.bold().underline().rgb(240, 105, 255).reset()
  }
  fn log(&self) -> String
  where
    Self: Display,
  {
    self.rgb(255, 253, 194).reset()
  }
}

impl Color for String {}
impl<'a> Color for &'a str {}

#[macro_export]
macro_rules! log {
  ( $($fn: ident).* @ $( $x: expr ),* ) => {
    {
      print!("{}", format!($($x),*).$($fn()).*);
    }
  };
  ( $( $x: expr ),* ) => {
    {
      print!("{}", format!($($x),*).log());
    }
  };
}
