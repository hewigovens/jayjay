use serde_json::json;
use worker::{Env, Method, Object, Request, Response, Result};

use super::package::{CHECKSUM_PREFIX, CHECKSUM_SUFFIX};
use super::{Package, SearchRequest};

/// Serves a WinGet REST source (`winget source add --type Microsoft.Rest`) for the ZIPs already on pkgs.hewig.dev.
pub(crate) async fn serve(mut req: Request, env: &Env) -> Result<Response> {
    let path = req.path();
    let route = path.strip_prefix("/winget/").unwrap_or_default();
    match (req.method(), route) {
        (Method::Get, "information") => Response::from_json(&json!({ "Data": {
            "SourceIdentifier": "pkgs.hewig.dev/winget",
            "ServerSupportedVersions": ["1.4.0", "1.5.0", "1.6.0", "1.7.0", "1.9.0", "1.10.0"],
            "UnsupportedPackageMatchFields": ["PackageFamilyName", "ProductCode", "UpgradeCode", "NormalizedPackageNameAndPublisher", "Market"],
            "RequiredPackageMatchFields": [],
            "UnsupportedQueryParameters": [],
            "RequiredQueryParameters": [],
            "Authentication": { "AuthenticationType": "none" },
        }})),
        (Method::Post, "manifestSearch") => {
            let Ok(request) = req.json::<SearchRequest>().await else {
                return Response::error("Invalid search request", 400);
            };
            let result = if request.matches() {
                load(env).await?.search_result()
            } else {
                None
            };
            match result {
                Some(result) => Response::from_json(&json!({ "Data": [result] })),
                None => Ok(Response::empty()?.with_status(204)),
            }
        }
        (Method::Get, route) if route.starts_with("packageManifests/") => {
            let id = route.trim_start_matches("packageManifests/");
            let url = req.url()?;
            let parameter = |key: &str| {
                url.query_pairs()
                    .find(|(name, _)| name == key)
                    .map(|(_, value)| value.into_owned())
            };
            let (version, channel) = (parameter("Version"), parameter("Channel"));
            match load(env)
                .await?
                .manifest(id, version.as_deref(), channel.as_deref())
            {
                Some(manifest) => Response::from_json(&json!({ "Data": manifest })),
                None => Response::from_json(
                    &json!({ "code": 404, "message": "No package manifest matches the request." }),
                )
                .map(|response| response.with_status(404)),
            }
        }
        _ => Response::error("Not Found", 404),
    }
}

async fn load(env: &Env) -> Result<Package> {
    let bucket = env.bucket("PACKAGES")?;
    let mut checksums = Vec::new();
    let mut cursor = None;
    loop {
        let mut list = bucket.list().prefix(CHECKSUM_PREFIX);
        if let Some(cursor) = cursor.take() {
            list = list.cursor(cursor);
        }
        let page = list.execute().await?;
        for object in page.objects() {
            let key = object.key();
            if !key.ends_with(CHECKSUM_SUFFIX) {
                continue;
            }
            if let Some(body) = bucket
                .get(&key)
                .execute()
                .await?
                .as_ref()
                .and_then(Object::body)
            {
                checksums.push((key, body.text().await?));
            }
        }
        cursor = page.cursor().filter(|_| page.truncated());
        if cursor.is_none() {
            return Ok(Package::from_checksums(checksums));
        }
    }
}
