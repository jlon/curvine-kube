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

use clap::Parser;
use curvine_kube::cli::{commands::Commands, CliArgs};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let args = CliArgs::parse();

    match args.command {
        Commands::Deploy(cmd) => cmd.execute().await,
        Commands::Update(cmd) => cmd.execute().await,
        Commands::List(cmd) => cmd.execute().await,
        Commands::Status(cmd) => cmd.execute().await,
        Commands::Delete(cmd) => cmd.execute().await,
    }
}
