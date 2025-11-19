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

//! Pod-related utilities and builders

pub mod builders;
pub mod merger;
pub mod template;
pub mod template_utils;

pub use self::builders::{EnvironmentBuilder, LifecycleBuilder, PodBuilder};
pub use self::merger::merge_pod_with_template;
pub use self::template::CurvinePod;
pub use self::template_utils::load_pod_from_template_file;
