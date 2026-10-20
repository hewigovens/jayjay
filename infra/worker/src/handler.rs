use chrono::DateTime;
use worker::{
    Context, Date, Env, Fetch, Headers, Method, Request, Response, ResponseBody, Result, event,
};

use crate::telemetry::Event;

#[event(fetch)]
pub async fn fetch(req: Request, env: Env, _ctx: Context) -> Result<Response> {
    let channel = if req.path() == "/ping" {
        "gpui"
    } else {
        "swiftui"
    };
    if let Some(event) = Event::from_url(&req.url()?, channel) {
        let _ = log_event(&env, &req, event).await;
    }
    if channel == "gpui" {
        Response::from_body(ResponseBody::Body(b"ok".to_vec()))
    } else {
        proxy_appcast(&env).await
    }
}

async fn proxy_appcast(env: &Env) -> Result<Response> {
    let origin = env
        .var("APPCAST_ORIGIN")
        .map(|value| value.to_string())
        .unwrap_or_default();
    let Ok(request) = Request::new(&origin, Method::Get) else {
        return Ok(Response::empty()?.with_status(500));
    };
    let Ok(response) = Fetch::Request(request).send().await else {
        return Ok(Response::empty()?.with_status(502));
    };
    let headers = Headers::new();
    headers.set("content-type", "application/xml; charset=utf-8")?;
    Response::from_body(response.body().clone()).map(|response| response.with_headers(headers))
}

async fn log_event(env: &Env, req: &Request, event: Event) -> Result<()> {
    let now = DateTime::from_timestamp_millis(Date::now().as_millis() as i64)
        .ok_or_else(|| worker::Error::RustError("invalid current timestamp".into()))?;
    let ip = req.headers().get("CF-Connecting-IP")?.unwrap_or_default();
    let secret = env
        .secret("HASH_SECRET")
        .map(|value| value.to_string())
        .unwrap_or_default();
    let (daily, monthly, identity_kind) = event.identity_keys(&ip, now, &secret);
    env.d1("DB")?
        .prepare(
            "INSERT INTO pings (day, unique_key, monthly_key, identity_kind, channel, platform, os, os_version, arch, version, build, model)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&[
            ((now.timestamp() / 86400) as f64).into(),
            daily.into(),
            monthly.into(),
            identity_kind.into(),
            event.channel.into(),
            event.platform.into(),
            event.os.into(),
            event.os_version.into(),
            event.arch.into(),
            event.version.into(),
            event.build.into(),
            event.model.into(),
        ])?
        .run()
        .await?;
    Ok(())
}
