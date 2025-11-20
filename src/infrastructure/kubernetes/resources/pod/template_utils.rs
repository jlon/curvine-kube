// Copyright 2025 JiangLong.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::infrastructure::kubernetes::resources::pod::template::CurvinePod;
use crate::shared::error::KubeError;
use k8s_openapi::api::core::v1::Pod;
use std::path::PathBuf;

pub fn load_pod_from_template_file(
    file_path: &str,
    main_container_name: &str,
) -> Result<CurvinePod, KubeError> {
    let path = resolve_pod_template_path(file_path)?;

    if !path.exists() {
        return Err(KubeError::ConfigError(format!(
            "Pod template file does not exist: {}",
            path.display()
        )));
    }

    let content = std::fs::read_to_string(&path).map_err(|e| {
        KubeError::ConfigError(format!(
            "Failed to read pod template file {}: {}",
            path.display(),
            e
        ))
    })?;

    let pod: Pod = serde_yaml::from_str(&content).map_err(|e| {
        KubeError::ConfigError(format!(
            "Failed to parse pod template file {}: {}",
            path.display(),
            e
        ))
    })?;

    if pod.spec.is_none() {
        return Err(KubeError::ConfigError(format!(
            "Pod template file {} is missing spec section",
            path.display()
        )));
    }

    // Validate that the main container exists in the template
    if let Some(ref spec) = pod.spec {
        let container_names: Vec<&str> = spec.containers.iter().map(|c| c.name.as_str()).collect();

        if !container_names.contains(&main_container_name) {
            return Err(KubeError::ValidationError(format!(
                "\n Container name mismatch in Pod template\n\
                \n  Expected container name: '{}'\n\
                Found container names: {}\n\
                \n The Pod template must contain a container named '{}'.\n\
                \n To fix this issue, update your Pod template:\n\
                \n  containers:\n\
                - name: {}  # Must match expected name\n\
                  # ... rest of container spec\n\
                \n  File: {}",
                main_container_name,
                container_names.join(", "),
                main_container_name,
                main_container_name,
                path.display()
            )));
        }
    }

    Ok(CurvinePod::new(pod, main_container_name))
}

pub fn resolve_pod_template_path(path: &str) -> Result<PathBuf, KubeError> {
    let path = PathBuf::from(path);

    if path.is_absolute() {
        Ok(path)
    } else {
        std::env::current_dir()
            .map_err(|e| KubeError::ConfigError(format!("Cannot get current directory: {}", e)))?
            .join(path)
            .canonicalize()
            .map_err(|e| KubeError::ConfigError(format!("Cannot resolve template path: {}", e)))
    }
}

pub(crate) fn format_bytes(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;
    const TB: u64 = GB * 1024;

    if bytes >= TB {
        format!("{}Ti", bytes / TB)
    } else if bytes >= GB {
        format!("{}Gi", bytes / GB)
    } else if bytes >= MB {
        format!("{}Mi", bytes / MB)
    } else if bytes >= KB {
        format!("{}Ki", bytes / KB)
    } else {
        format!("{}", bytes)
    }
}
