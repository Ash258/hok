mod search;

pub use search::search;

use crate::manager::BucketManager;
use crate::model::App;
use crate::model::Manifest;
use crate::Config;
use crate::ScoopResult;

/// Find and return local manifest represented as [`Manifest`], using given
/// `pattern`.
pub fn find_manifest<S>(config: &Config, pattern: S) -> ScoopResult<Option<Manifest>>
where
    S: AsRef<str>,
{
    let bucket_manager = BucketManager::new(config);

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
            let bucket = bucket_manager.bucket(bucket_name).unwrap();
            let manifest_path = bucket.manifest_dir().join(format!("{}.json", app_name));
            match manifest_path.exists() {
                true => Ok(Some(Manifest::new(&manifest_path)?)),
                false => Ok(None),
            }
        }
        None => {
            for bucket in bucket_manager.buckets().iter() {
                let manifest_path = bucket.manifest_dir().join(format!("{}.json", app_name));
                if manifest_path.exists() {
                    return Ok(Some(Manifest::new(&manifest_path)?));
                }
            }

            Ok(None)
        }
    }
}

pub fn find_app<'cfg, S>(config: &'cfg Config, pattern: S) -> ScoopResult<Option<App<'cfg>>>
where
    S: AsRef<str>,
{
    let pattern = pattern.as_ref();
    let bucket_manager = BucketManager::new(config);
    // Cehck the given pattern whether having bucket name prefix
    let (bucket_name, app_name) = match pattern.contains("/") {
        true => pattern.split_once("/").map(|(a, b)| (Some(a), b)).unwrap(),
        false => (None, pattern),
    };
    match bucket_name {
        Some(bucket_name) => {
            let bucket = bucket_manager.bucket(bucket_name).unwrap();
            return bucket.app(app_name).map(|app| Some(app));
        }
        None => {
            for bucket in bucket_manager.buckets().into_iter() {
                if bucket.contains_app(app_name) {
                    return bucket.app(app_name).map(|app| Some(app));
                }
            }
        }
    }
    Ok(None)
}