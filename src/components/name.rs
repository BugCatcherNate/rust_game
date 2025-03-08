#[derive(Debug, Clone, PartialEq)]
pub struct Name(pub String);


impl PartialEq<&Name> for Name {
    fn eq(&self, other: &&Name) -> bool {
        &self.0 == &other.0
    }
}


