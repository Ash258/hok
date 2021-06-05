use anyhow::{anyhow, Result};
use std::fs::File;
use std::io::BufReader;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::{fs, spdx, Scoop};

#[derive(Debug)]
pub enum ManifestKind {
    Local(PathBuf),
    Remote(String),
}

#[derive(Debug)]
pub struct Manifest {
    pub app: String,
    pub bucket: Option<String>,
    pub version: String,
    pub license: Option<Vec<(String, Option<String>)>>,
    pub json: Value,
    pub kind: ManifestKind,
}

impl Manifest {
    /// Create an [`Manifest`] from the given [`PathBuf`].
    pub fn from_path<P: AsRef<Path> + ?Sized>(
        path: &P,
        bucket: Option<String>,
    ) -> Result<Manifest> {
        let buf = BufReader::new(File::open(path.as_ref())?);

        match serde_json::from_reader(buf) {
            Ok(v) => {
                let json: Value = v;
                let version = json.get("version");
                if version.is_none() {
                    let msg = format!(
                        "Failed to read version from manifest '{}'",
                        path.as_ref().to_str().unwrap()
                    );
                    return Err(anyhow!(msg));
                }

                return Ok(Manifest {
                    app: fs::leaf_base(path.as_ref()).to_string(),
                    bucket,
                    version: version.unwrap().as_str().unwrap().to_string(),
                    license: Self::license(json.get("license")),
                    json,
                    kind: ManifestKind::Local(path.as_ref().to_path_buf()),
                });
            }
            Err(_e) => {
                let msg = format!(
                    "Failed to parse manifest '{}'",
                    path.as_ref().to_str().unwrap()
                );
                return Err(anyhow!(msg));
            }
        }
    }

    pub fn from_url<T: AsRef<str>>(_url: T) -> Result<Manifest> {
        todo!()
    }

    /// Extract license pair from the JSON `license` field
    fn license(val: Option<&Value>) -> Option<Vec<(String, Option<String>)>> {
        let generator = |license| -> (String, Option<String>) {
            let url = match license {
                "Freeware" => Some("https://en.wikipedia.org/wiki/Freeware".to_string()),
                "Public Domain" => {
                    Some("https://en.wikipedia.org/wiki/Public_domain_software".to_string())
                }
                "Shareware" => Some("https://en.wikipedia.org/wiki/Shareware".to_string()),
                "Proprietary" => {
                    Some("https://en.wikipedia.org/wiki/Proprietary_software".to_string())
                }
                "Unknown" => None,
                license => match spdx::SPDX.contains(license) {
                    true => Some(format!("https://spdx.org/licenses/{}.html", license)),
                    false => None,
                },
            };
            (license.to_string(), url)
        };

        match val {
            Some(Value::String(str)) => {
                let mut license_pair: Vec<(String, Option<String>)> = vec![];
                if str.contains("|") {
                    str.split("|")
                        .filter(|s| !(*s).eq("..."))
                        .for_each(|s| license_pair.push(generator(s)));
                } else if str.contains(",") {
                    str.split(",")
                        .filter(|s| !(*s).eq("..."))
                        .for_each(|s| license_pair.push(generator(s)));
                } else {
                    license_pair.push(generator(str));
                }
                return Some(license_pair);
            }
            Some(Value::Object(pair)) => {
                if pair.get("identifier").is_none() {
                    return None;
                }
                let license = pair.get("identifier").unwrap().to_string();
                let url = match pair.get("url") {
                    Some(url) => Some(url.to_string()),
                    None => None,
                };
                return Some(vec![(license, url)]);
            }
            _ => None,
        }
    }
}

impl<'a> Scoop<'a> {
    /// Find and return local manifest represented as [`ScoopAppManifest`],
    /// using given `pattern`.
    ///
    /// bucket name prefix is support, for example:
    /// ```
    /// find_local_manifest("main/gcc")
    /// ```
    pub fn find_local_manifest<T: AsRef<str>>(&mut self, pattern: T) -> Result<Option<Manifest>> {
        // Detect given pattern whether having bucket name prefix
        let (bucket_name, app_name) = match pattern.as_ref().contains("/") {
            true => {
                let (a, b) = pattern.as_ref().split_once("/").unwrap();
                (Some(a), b)
            }
            false => (None, pattern.as_ref()),
        };

        match bucket_name {
            Some(bucket_name) => {
                let bucket = self.bucket_manager.get_bucket(bucket_name).unwrap();
                let manifest_path = bucket.manifest_dir().join(format!("{}.json", app_name));
                match manifest_path.exists() {
                    true => Ok(Some(Manifest::from_path(
                        &manifest_path,
                        Some(bucket.name.to_string()),
                    )?)),
                    false => Ok(None),
                }
            }
            None => {
                for bucket in self.bucket_manager.get_buckets() {
                    let manifest_path = bucket.1.manifest_dir().join(format!("{}.json", app_name));
                    match manifest_path.exists() {
                        true => {
                            return Ok(Some(Manifest::from_path(
                                &manifest_path,
                                Some(bucket.1.name.to_string()),
                            )?))
                        }
                        false => {}
                    }
                }

                Ok(None)
            }
        }
    }

    // Deprecated, will be replaced by ScoopAppManifest::from_url()
    // #[deprecated]
    // pub fn manifest_from_url(&self, manifest_url: &str) -> Result<Value> {
    //   // Use proxy from Scoop's config
    //   let agent = match self.config["proxy"].clone() {
    //     Value::String(mut proxy) => {
    //       if !proxy.starts_with("http") {
    //         proxy.insert_str(0, "http://");
    //       }

    //       let proxy = ureq::Proxy::new(proxy)?;

    //       ureq::AgentBuilder::new()
    //         .proxy(proxy)
    //         .build()
    //     },
    //     _ => {
    //       ureq::AgentBuilder::new()
    //         .build()
    //     }
    //   };

    //   let body: serde_json::Value = agent.get(manifest_url)
    //     .call()?
    //     .into_json()?;

    //   Ok(body)
    // }
}