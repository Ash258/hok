use reqwest::{Client as ReqwestClient, Method, IntoUrl, Proxy, RequestBuilder, Result};

use crate::config::Config;

static SCOOP_USER_AGENT: &str = "Scoop/0.1.0 (Rust)";

#[derive(Debug)]
pub struct Client {
  inner: ReqwestClient,
}

impl Default for Client {
  fn default() -> Self {
      Self::new(&Config::default()).unwrap()
  }
}

impl Client {
  pub fn new(config: &Config) -> Result<Self> {
    let proxy = config.get("proxy");
    let mut builder = ReqwestClient::builder()
      .user_agent(SCOOP_USER_AGENT);
    // Add proxy
    if proxy.is_some() {
      let mut proxy = proxy.unwrap().as_str().unwrap().to_string();
      if !proxy.starts_with("http") {
        proxy.insert_str(0, "http://");
      }

      builder = builder.proxy(Proxy::all(proxy)?)
    }

    Ok(Client {
      inner: builder.build()?
    })
  }

  pub fn get<U: IntoUrl>(&self, url: U) -> RequestBuilder {
    self.inner.request(Method::GET, url)
  }

  pub fn post<U: IntoUrl>(&self, url: U) -> RequestBuilder {
    self.inner.request(Method::POST, url)
  }

  pub fn head<U: IntoUrl>(&self, url: U) -> RequestBuilder {
    self.inner.request(Method::HEAD, url)
  }
}