//! The gateway's router: `model_list` built through router-core's deployment build, and
//! `router_settings` read by the core. The config load resolved every `os.environ/` reference.

use crate::host::NativeBootHost;
use litellm_config::Config;
use litellm_router_core::{BuildPath, Router, RouterConfig, build::Install};
use serde_json::{Map, Value};

use crate::Error;

fn written(value: &impl serde::Serialize) -> Map<String, Value> {
    match serde_json::to_value(value) {
        Ok(Value::Object(fields)) => fields,
        _ => Map::new(),
    }
}

fn entries(config: &Config) -> Vec<Map<String, Value>> {
    config
        .model_list
        .iter()
        .map(|model| {
            let params = written(&model.written_params);
            written(&*model.additional_fields)
                .into_iter()
                .chain([
                    (
                        "model_name".to_owned(),
                        Value::from(model.model_name.as_str()),
                    ),
                    ("litellm_params".to_owned(), Value::Object(params)),
                    (
                        "model_info".to_owned(),
                        Value::Object(written(&*model.model_info)),
                    ),
                ])
                .collect()
        })
        .collect()
}

/// The router `config` describes.
pub(crate) async fn router(config: &Config) -> Result<Router, Error> {
    let (settings, dynamic) = RouterConfig::from_settings(&config.router_settings)?;
    let router = Router::new(settings, dynamic)?;
    router
        .build(
            &NativeBootHost,
            BuildPath::Create,
            entries(config),
            true,
            Install::Replace,
        )
        .await?;
    Ok(router)
}

/// The router `config` describes with no host behind its build, for a test that asks it only what
/// its settings and model groups say.
#[cfg(test)]
pub(crate) fn unhosted(config: &Config) -> Router {
    let (settings, dynamic) = RouterConfig::from_settings(&config.router_settings).unwrap();
    let model_list = entries(config)
        .into_iter()
        .map(|entry| serde_json::from_value(Value::Object(entry)).unwrap())
        .collect();
    let settings = RouterConfig {
        model_list,
        ..settings
    };
    Router::new(settings, dynamic).unwrap()
}
