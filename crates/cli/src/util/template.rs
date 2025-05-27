use florca_core::provider::Provider;
use itertools::Itertools;
use rust_embed::Embed;
use std::{borrow::Cow, collections::HashMap, sync::LazyLock};

pub use functions::available_function_templates;
pub use functions::get_function_template;
pub use plugin::get_plugin_template;

#[derive(Embed)]
#[folder = "../../templates/"]
struct TemplatesAsset;

mod plugin {
    use super::TemplatesAsset;

    /// # Panics
    ///
    /// Panics if the template for plugins cannot be found.
    #[must_use]
    pub fn get_plugin_template() -> Vec<u8> {
        TemplatesAsset::get("plugin.ts")
            .expect("Plugin template not found")
            .data
            .into_owned()
    }
}

mod functions {
    use super::{Cow, HashMap, Itertools, LazyLock, Provider, TemplatesAsset};

    pub fn available_function_templates() -> HashMap<Provider, Vec<String>> {
        FUNCTION_TEMPLATES
            .iter()
            .map(|template| (template.provider, template.runtime.clone()))
            .into_group_map()
    }

    pub fn get_function_template(
        provider: Provider,
        runtime: &str,
    ) -> Option<Vec<FunctionTemplateFile>> {
        FUNCTION_TEMPLATES
            .iter()
            .find(|template| template.provider == provider && template.runtime == runtime)
            .map(FunctionTemplate::files)
    }

    pub struct FunctionTemplateFile {
        pub relative_file_path: String,
        pub bytes: Cow<'static, [u8]>,
    }

    static FUNCTION_TEMPLATES: LazyLock<Vec<FunctionTemplate>> = LazyLock::new(|| {
        let mut map: HashMap<Provider, HashMap<String, Vec<String>>> = HashMap::new();
        for file in TemplatesAsset::iter() {
            if let Some((provider, runtime, _suffix)) = file.split('/').collect_tuple::<(_, _, _)>()
            {
                map.entry(provider.into())
                    .or_default()
                    .entry(runtime.into())
                    .or_default()
                    .push(file.to_string());
            }
        }
        map.iter()
            .flat_map(|(provider, runtimes)| {
                runtimes
                    .iter()
                    .map(move |(runtime, embed_file_paths)| FunctionTemplate {
                        provider: *provider,
                        runtime: runtime.into(),
                        embed_file_paths: embed_file_paths.clone(),
                    })
            })
            .collect()
    });

    #[derive(Debug)]
    struct FunctionTemplate {
        provider: Provider,
        runtime: String,
        embed_file_paths: Vec<String>,
    }

    impl FunctionTemplate {
        fn files(&self) -> Vec<FunctionTemplateFile> {
            self.embed_file_paths
                .iter()
                .map(|path| {
                    let embedded_file = TemplatesAsset::get(path).expect("Invalid file path");
                    let relative_file_path = path.split('/').skip(2).join("/");
                    FunctionTemplateFile {
                        relative_file_path,
                        bytes: embedded_file.data,
                    }
                })
                .collect()
        }
    }
}
