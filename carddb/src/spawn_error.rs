use nom::error::VerboseError;

use texttoken::Tokens;

#[derive(Debug, Clone)]
pub enum SpawnError<'a> {
    Nom(nom::Err<VerboseError<&'a Tokens>>),
    CardNotFoundError(&'a str),
}
impl<'a> From<nom::Err<VerboseError<&'a Tokens>>> for SpawnError<'a> {
    fn from(err: nom::Err<VerboseError<&'a Tokens>>) -> Self {
        Self::Nom(err)
    }
}
