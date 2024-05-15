use std::{error::Error, fmt};

#[derive(Debug, Clone)]
pub struct TagNotFoundError(pub String);
impl Error for TagNotFoundError {}
impl fmt::Display for TagNotFoundError {
	fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    	write!(f, "Tag did not match! Please check you spelling or implement the tag in src/lib.rs in get_content().\nTAG: {}", &self.0)
    }
}
