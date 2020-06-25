use rocket::request::{FromRequest, Outcome};
use rocket::Request;

pub struct WebLanguage(String);

impl<'a, 'r> FromRequest<'a, 'r> for WebLanguage {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> Outcome<Self, Self::Error> {
        let languages = crate::res::words::languages();
        if let Some(cookie) = request.cookies().get("language") {
            let value = cookie.value().to_string();
            if languages.contains(&value) {
                return Outcome::Success(WebLanguage(value));
            }
        }
        Outcome::Success(WebLanguage("english".into()))
    }
}