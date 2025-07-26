#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct User {
    email: String,
    password: String,
    requires_2fa: bool,
}

impl User {
    pub fn new(email: String, password: String, requires_2fa: bool) -> Self {
        Self {
            email,
            password,
            requires_2fa,
        }
    }

    pub fn email(&self) -> &String {
        &self.email
    }

    pub fn password(&self) -> &String {
        &self.password
    }

    pub fn requires_2fa(&self) -> bool {
        self.requires_2fa
    }
}
