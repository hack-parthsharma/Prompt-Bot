use std::{error, result};

pub const ROW_SEP: char = ':';
pub const BUTTON_SEP: char = ',';
pub type Result<T> = result::Result<T, Box<dyn error::Error>>;
