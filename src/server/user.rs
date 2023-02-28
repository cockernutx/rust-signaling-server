use serde::Deserialize;

#[derive(Deserialize, PartialEq)]
pub struct User {
    username: String,
    password: String
}

 impl User {
   pub  fn new(username: String, password: String) -> Self {
        User { username: username, password: password }
    }
}
