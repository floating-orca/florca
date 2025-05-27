use reqwest::Url;
use secrecy::SecretString;
use std::env;

#[derive(Debug, Clone)]
pub struct BasicAuth {
    pub username: String,
    pub password: Option<SecretString>,
}

impl BasicAuth {
    pub fn new(username: String, password: Option<String>) -> Self {
        Self {
            username,
            password: password.map(SecretString::from),
        }
    }

    /// # Panics
    ///
    /// Panics if `BASIC_AUTH_USERNAME` is not set or not valid unicode or if `BASIC_AUTH_PASSWORD` is not valid unicode
    #[must_use]
    pub fn from_env() -> Self {
        let username = env::var("BASIC_AUTH_USERNAME")
            .expect("BASIC_AUTH_USERNAME is not set or not valid unicode");
        let password = match env::var("BASIC_AUTH_PASSWORD") {
            Ok(password) => Some(password),
            Err(env::VarError::NotPresent) => None,
            Err(env::VarError::NotUnicode(_)) => panic!("BASIC_AUTH_PASSWORD is not valid unicode"),
        };
        Self::new(username, password)
    }

    #[must_use]
    pub fn expose_password(&self) -> Option<&str> {
        self.password
            .as_ref()
            .map(secrecy::ExposeSecret::expose_secret)
    }
}

#[derive(Debug, Clone)]
pub struct DeployerUrl;

impl DeployerUrl {
    /// # Panics
    ///
    /// Panics if `DEPLOYER_URL` environment variable is not set or not valid unicode
    #[must_use]
    pub fn base() -> Url {
        let mut url =
            env::var("DEPLOYER_URL").expect("DEPLOYER_URL is not set or not valid unicode");
        if !url.ends_with('/') {
            url.push('/');
        }
        Url::parse(&url).expect("DEPLOYER_URL is not a valid URL")
    }

    /// # Panics
    ///
    /// Panics if `DEPLOYER_URL` environment variable is not set or not valid unicode or if the URL
    /// cannot be a base
    #[must_use]
    pub fn path(segments: &[&str]) -> Url {
        let mut url = Self::base();
        url.path_segments_mut()
            .expect("DEPLOYER_URL cannot be a base URL")
            .extend(segments);
        url
    }
}

#[derive(Debug, Clone)]
pub struct EngineUrl;

impl EngineUrl {
    /// # Panics
    ///
    /// Panics if `ENGINE_URL` environment variable is not set or not valid unicode
    #[must_use]
    pub fn base() -> Url {
        let mut url = env::var("ENGINE_URL").expect("ENGINE_URL is not set or not valid unicode");
        if !url.ends_with('/') {
            url.push('/');
        }
        Url::parse(&url).expect("ENGINE_URL is not a valid URL")
    }

    /// # Panics
    ///
    /// Panics if `ENGINE_URL` environment variable is not set or not valid unicode or if the URL
    /// cannot be a base
    #[must_use]
    pub fn path(segments: &[&str]) -> Url {
        let mut url = Self::base();
        url.path_segments_mut()
            .expect("ENGINE_URL cannot be a base URL")
            .extend(segments);
        url
    }
}

pub trait RequestBuilderExt {
    #[must_use]
    fn with_basic_auth_from_env(self) -> Self;
}

impl RequestBuilderExt for reqwest::RequestBuilder {
    fn with_basic_auth_from_env(self) -> Self {
        let basic_auth = BasicAuth::from_env();
        self.basic_auth(&basic_auth.username, basic_auth.expose_password())
    }
}

impl RequestBuilderExt for reqwest::blocking::RequestBuilder {
    fn with_basic_auth_from_env(self) -> Self {
        let basic_auth = BasicAuth::from_env();
        self.basic_auth(&basic_auth.username, basic_auth.expose_password())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_expose_password() {
        let auth = BasicAuth::new("user".to_string(), Some("pass".to_string()));
        assert_eq!(auth.expose_password(), Some("pass"));
    }

    #[test]
    fn test_expose_password_none() {
        let auth = BasicAuth::new("user".to_string(), None);
        assert_eq!(auth.expose_password(), None);
    }
}
