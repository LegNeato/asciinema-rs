use failure::Error;
use url::Url;

#[derive(Clone, Debug)]
pub struct Api {
    authentication_url: Url,
    base_url: Url,
    upload_url: Url,
}

impl Api {
    pub fn new(base_url: Url) -> Result<Self, Error> {
        // The `Url` trait treats trailing slashes as significant:
        // <https://docs.rs/url/*/url/struct.Url.html#method.join>
        // We make sure there is always a trailing slash so the joins
        // below behave correctly.
        let mut base_raw = base_url.to_string();
        if let Some(last_char) = base_raw.pop() {
            if last_char != '/' {
                base_raw.push(last_char);
                base_raw.push('/');
            } else {
                base_raw.push(last_char);
            }
        }
        let normalized_base = Url::parse(&base_raw)?;

        let authentication_url = normalized_base.join("connect/")?;
        let upload_url = normalized_base.join("api/asciicasts")?;
        Ok(Api {
            authentication_url,
            base_url: normalized_base,
            upload_url,
        })
    }
    pub fn authentication_url(self) -> Url {
        self.authentication_url
    }
    pub fn base_url(self) -> Url {
        self.base_url
    }
    pub fn upload_url(self) -> Url {
        self.upload_url
    }
}

impl Default for Api {
    fn default() -> Self {
        Api::new(Url::parse("http://asciinema.org").unwrap()).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use url::Url;

    #[test]
    fn base_url() {
        let base = Url::parse("http://www.example.com").unwrap();
        let a = Api::new(base.clone()).unwrap();
        assert_eq!(a.base_url(), base);
    }

    #[test]
    fn normalized_base_url() {
        let base = Url::parse("http://www.example.com/bar").unwrap();
        let a = Api::new(base.clone()).unwrap();
        assert_eq!(a.base_url().to_string(), format!("{}/", base));
    }

    #[test]
    fn authentication_url() {
        let base = Url::parse("http://www.example.com").unwrap();
        let a = Api::new(base).unwrap();
        assert_eq!(
            a.authentication_url().as_str(),
            "http://www.example.com/connect/"
        );
    }

    #[test]
    fn normalized_authentication_url() {
        let base = Url::parse("http://www.example.com/whatever").unwrap();
        let a = Api::new(base).unwrap();
        assert_eq!(
            a.authentication_url().as_str(),
            "http://www.example.com/whatever/connect/"
        );
    }

    #[test]
    fn upload_url() {
        let base = Url::parse("http://www.example.com").unwrap();
        let a = Api::new(base).unwrap();
        assert_eq!(
            a.upload_url().as_str(),
            "http://www.example.com/api/asciicasts"
        );
    }

    #[test]
    fn normalized_upload_url() {
        let base = Url::parse("http://www.example.com/blah").unwrap();
        let a = Api::new(base).unwrap();
        assert_eq!(
            a.upload_url().as_str(),
            "http://www.example.com/blah/api/asciicasts"
        );
    }
}
