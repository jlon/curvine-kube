# curvine-kube

**curvine-kube** 是一个用于在 Kubernetes 上部署和管理 Curvine 分布式存储集群的独立 CLI 工具。

## ✨ 特性

- 🚀 **一键部署**：快速在 Kubernetes 上部署完整的 Curvine 集群
- 🔄 **动态更新**：支持在线更新集群配置、镜像和副本数
- 📊 **状态监控**：实时查看集群状态和资源使用情况
- 🎯 **灵活配置**：支持通过配置文件和命令行参数灵活配置
- 🔧 **自动化运维**：自动生成 ConfigMap、Service、StatefulSet 等 K8s 资源
- 🏗️ **架构清晰**：采用 DDD 分层架构，易于维护和扩展

## 📋 前置要求

- Rust 1.90 或更高版本
- Kubernetes 集群（1.20+）
- kubectl 已配置并可访问目标集群
- Curvine 集群配置文件（`curvine-cluster.toml`）

## 🔧 安装

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/jlon/curvine-kube.git
cd curvine-kube

# 构建 Release 版本
cargo xtask build --release

# 安装到系统
cargo xtask install
# 或指定安装路径
cargo xtask install --prefix ~/.local
```

### 使用 cargo install

```bash
cargo install --path .
```

### 使用 cargo-binstall

```bash
cargo binstall curvine-kube
```

## 🚀 快速开始

### 1. 准备配置文件

创建 `curvine-cluster.toml` 配置文件：

```toml
cluster_id = "my-cluster"

[master]
meta_dir = "data/meta"
rpc_port = 8995

[journal]
enable = true
journal_dir = "data/journal"
rpc_port = 8996

[worker]
data_dir = [
    "[MEM:10GB]/data/mem",
    "[SSD:100GB]/data/ssd"
]
rpc_port = 8997

[client]
block_size_str = "64MB"
```

### 2. 部署集群

```bash
# 设置配置文件环境变量
export CURVINE_CONF_FILE=/path/to/curvine-cluster.toml

# 部署集群
curvine-kube deploy -c my-cluster \
  --namespace curvine \
  --master-replicas 3 \
  --worker-replicas 5 \
  --image docker.io/curvine:v1.0.0 \
```

### 3. 查看集群状态

```bash
# 查看集群状态
curvine-kube status my-cluster -n curvine

# 列出所有集群
curvine-kube list
```

### 4. 更新集群

```bash
# 更新 Worker 副本数
curvine-kube update -c my-cluster \
  --worker-replicas 10

# 更新镜像版本
curvine-kube update -c my-cluster \
  --image docker.io/curvine:v1.0.0 \
```

### 5. 删除集群

```bash
# 删除集群（保留 PVC）
curvine-kube delete my-cluster -n curvine

# 删除集群和所有持久化数据
curvine-kube delete my-cluster -n curvine --delete-pvcs
```

## 📖 详细用法

### 部署命令

```bash
curvine-kube deploy [OPTIONS]

选项:
  -c, --cluster-id <ID>              集群 ID（必需）
  -n, --namespace <NS>               Kubernetes 命名空间 [默认: default]
      --config-file <FILE>           配置文件路径
      --master-replicas <N>          Master 副本数 [默认: 3]
      --worker-replicas <N>          Worker 副本数 [默认: 3]
      --image <IMAGE>                Master、Worker 镜像
      --storage-class <CLASS>        Master、Worker的StorageClass 名称
      --service-type <TYPE>          Service 类型 [默认: ClusterIP]
  -D <KEY=VALUE>                     动态配置参数
```

### 动态配置参数

通过 `-D` 参数可以覆盖配置文件中的设置：

```bash
curvine-kube deploy -c my-cluster \
  -Dkubernetes.master.cpu=2.0 \
  -Dkubernetes.master.memory=4Gi \
  -Dkubernetes.worker.cpu=4.0 \
  -Dkubernetes.worker.memory=8Gi \
  -Dkubernetes.worker.labels=tier=storage,env=prod
```

支持的参数：

- `kubernetes.master.cpu` / `kubernetes.worker.cpu`
- `kubernetes.master.memory` / `kubernetes.worker.memory`
- `kubernetes.master.labels` / `kubernetes.worker.labels`
- `kubernetes.master.annotations` / `kubernetes.worker.annotations`
- `kubernetes.master.node-selector` / `kubernetes.worker.node-selector`

### 环境变量

- `CURVINE_CONF_FILE`：配置文件路径
- `KUBECONFIG`：Kubernetes 配置文件路径

## 🏗️ 架构设计

项目采用清晰的分层架构：

```text
src/
├── cli/                    # CLI 命令层
├── domain/                 # 领域层
│   ├── cluster/           # 集群管理
│   └── config/            # 配置管理
├── infrastructure/         # 基础设施层
│   └── kubernetes/        # K8s 资源构建
└── shared/                # 共享工具层
```

### 核心组件

- **CurvineClusterDescriptor**：集群生命周期管理
- **KubernetesValidator**：配置验证
- **ConfigMapBuilder**：动态生成集群配置
- **StatefulSet Builders**：构建 Master 和 Worker 资源

## 🔨 开发

### 构建系统

项目使用 `cargo xtask` 构建系统：

```bash
# 查看所有命令
cargo xtask --help

# 构建项目
cargo xtask build

# 运行测试
cargo xtask test

# 运行 CI 检查
cargo xtask ci

# 创建发布包
cargo xtask dist
```

或使用 Makefile：

```bash
make help        # 查看所有命令
make build       # 构建
make test        # 测试
make ci          # CI 检查
```

### 运行测试

```bash
# 运行所有测试
cargo test --all

# 运行集成测试
cargo test --test '*'

# 运行特定测试
cargo test test_worker_mem_volume_creation
```

### 代码质量

```bash
# 格式化代码
cargo fmt --all

# 运行 Clippy
cargo clippy --all-targets --all-features

# 完整 CI 检查
cargo xtask ci
```

## 📦 发布优化

项目使用以下 Release 优化配置：

- **LTO (Link-Time Optimization)**：链接时优化
- **Strip symbols**：移除调试符号
- **Codegen units = 1**：最大化优化
- **Opt-level = 3**：最高优化级别

这些配置可显著减小二进制大小并提升运行性能。

## 🤝 贡献

欢迎贡献！请遵循以下步骤：

1. Fork 本仓库
2. 创建特性分支 (`git checkout -b feature/amazing-feature`)
3. 提交更改 (`git commit -m 'Add amazing feature'`)
4. 推送到分支 (`git push origin feature/amazing-feature`)
5. 开启 Pull Request

### 贡献指南

- 遵循 Rust 代码规范
- 添加必要的测试
- 更新相关文档
- 确保 CI 检查通过

## 📄 许可证

本项目采用 Apache License 2.0 许可证 - 详见 [LICENSE](LICENSE) 文件

## 📮 联系方式

- 问题反馈：[GitHub Issues](https://github.com/jlon/curvine-kube/issues)
- 讨论交流：[GitHub Discussions](https://github.com/jlon/curvine-kube/discussions)

---

**注意**：本工具仅用于 Kubernetes 部署管理，不包含 Curvine 核心功能。
