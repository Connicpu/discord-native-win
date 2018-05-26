
#[derive(Serialize)]
pub struct PasswordLoginRequest<'a> {
    email: &'a str,
    password: &'a str,
}
