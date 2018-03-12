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
        let authentication_url = base_url.join("/connect/")?;
        let upload_url = base_url.join("/api/asciicasts")?;
        Ok(Api {
            authentication_url,
            base_url,
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
    fn authentication_url() {
        let base = Url::parse("http://www.example.com").unwrap();
        let a = Api::new(base).unwrap();
        assert_eq!(
            a.authentication_url().as_str(),
            "http://www.example.com/connect/"
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
}
