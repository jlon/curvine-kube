# Curvine-Kube 生产环境 E2E 测试方案

## 测试目标

从生产环境的角度，全面测试 curvine-kube 工具的所有功能，确保其能够在生产环境中稳定、可靠地运行。

## 测试原则

1. **生产环境优先**：测试场景必须贴近真实生产环境
2. **全面覆盖**：覆盖所有可配置参数和功能特性
3. **高可用验证**：重点测试 HA 场景和故障恢复
4. **配置验证**：不仅测试功能，还要验证配置是否正确应用
5. **边界测试**：测试异常情况和边界条件

## 测试模块规划

### 模块 1: 基础部署测试 (Basic Deployment)

#### 1.1 单副本部署（开发环境）
- **目标**：验证最小化部署
- **参数**：
  - Master replicas: 1
  - Worker replicas: 1
  - Service type: ClusterIP
- **验证点**：
  - 所有资源创建成功
  - Pod 正常运行
  - Service 可访问

#### 1.2 多副本部署（生产环境）
- **目标**：验证生产环境标准部署
- **参数**：
  - Master replicas: 3（奇数，Raft 要求）
  - Worker replicas: 5
  - Service type: ClusterIP
- **验证点**：
  - Master 奇数副本验证
  - 所有 Pod 正常运行
  - 服务发现正常

#### 1.3 大规模部署
- **目标**：验证大规模集群部署
- **参数**：
  - Master replicas: 5
  - Worker replicas: 10
- **验证点**：
  - 所有 Pod 正常启动
  - 资源使用合理
  - 性能稳定

### 模块 2: 资源管理测试 (Resource Management)

#### 2.1 CPU/内存限制配置
- **目标**：验证资源限制正确应用
- **参数**：
  - Master CPU: 2.0, Memory: 4Gi
  - Worker CPU: 1.0, Memory: 2Gi
- **验证点**：
  - StatefulSet/Deployment 中资源限制正确
  - Requests = Limits（生产推荐）
  - Pod 可以正常启动

#### 2.2 资源请求配置
- **目标**：验证资源请求配置
- **参数**：
  - 使用 -D 参数设置资源
- **验证点**：
  - 资源请求正确应用
  - 调度器可以正确调度

#### 2.3 资源配额验证
- **目标**：验证资源配额限制
- **场景**：在有限制的命名空间中部署
- **验证点**：
  - 资源配额检查
  - 超出配额时的错误处理

### 模块 3: 高可用性测试 (High Availability)

#### 3.1 Master 奇数副本验证
- **目标**：验证 Raft 共识要求
- **测试场景**：
  - 部署偶数副本（应该失败）
  - 部署奇数副本（应该成功）
- **验证点**：
  - 偶数副本被拒绝
  - 奇数副本正常部署

#### 3.2 Worker 扩缩容
- **目标**：验证动态扩缩容
- **测试场景**：
  - 从 3 扩展到 10
  - 从 10 缩减到 5
- **验证点**：
  - 扩容成功
  - 缩容成功
  - 数据不丢失

#### 3.3 Pod 故障恢复
- **目标**：验证 Pod 自动恢复
- **测试场景**：
  - 删除 Master Pod
  - 删除 Worker Pod
- **验证点**：
  - Pod 自动重建
  - 服务不中断
  - 数据持久化

### 模块 4: 存储配置测试 (Storage Configuration)

#### 4.1 不同 StorageClass
- **目标**：验证存储类配置
- **参数**：
  - Master storage class: fast-ssd
  - Worker storage class: standard
- **验证点**：
  - PVC 使用正确的 StorageClass
  - 存储正确挂载

#### 4.2 存储大小配置
- **目标**：验证存储大小
- **参数**：
  - Master size: 50Gi
  - Worker size: 100Gi
- **验证点**：
  - PVC 大小正确
  - 存储可用

#### 4.3 PVC 管理
- **目标**：验证 PVC 删除选项
- **测试场景**：
  - 删除集群（保留 PVC）
  - 删除集群（删除 PVC）
- **验证点**：
  - PVC 正确保留/删除
  - 数据安全

### 模块 5: 网络配置测试 (Networking)

#### 5.1 Service 类型测试
- **目标**：验证不同 Service 类型
- **测试场景**：
  - ClusterIP（默认）
  - NodePort
  - LoadBalancer（如果支持）
- **验证点**：
  - Service 类型正确
  - 端口正确暴露
  - 服务可访问

#### 5.2 端口配置验证
- **目标**：验证端口配置
- **验证点**：
  - RPC 端口（8995）
  - Journal 端口（8996）
  - Web 端口（9000, 9001）
  - Worker 端口（8997）

#### 5.3 服务发现
- **目标**：验证 Kubernetes DNS 服务发现
- **验证点**：
  - Master DNS 名称正确
  - 服务间通信正常

### 模块 6: 配置管理测试 (Configuration Management)

#### 6.1 动态配置参数（-D flags）
- **目标**：验证动态配置功能
- **测试参数**：
  ```bash
  -Dkubernetes.master.cpu=2.0
  -Dkubernetes.master.memory=4Gi
  -Dkubernetes.worker.cpu=1.0
  -Dkubernetes.worker.memory=2Gi
  -Dkubernetes.master.labels=app=curvine,tier=master
  -Dkubernetes.master.annotations=prometheus.io/scrape=true
  -Dkubernetes.service.type=NodePort
  ```
- **验证点**：
  - 所有参数正确应用
  - 资源限制正确
  - Labels/Annotations 正确
  - Service 类型正确

#### 6.2 环境变量配置
- **目标**：验证环境变量
- **参数**：
  - `-Dkubernetes.master.env.JAVA_OPTS=-Xmx4g`
  - `-Dkubernetes.worker.env.CACHE_SIZE=10GB`
- **验证点**：
  - 环境变量正确设置
  - Pod 中可以访问

#### 6.3 Labels 和 Annotations
- **目标**：验证标签和注解
- **参数**：
  - Master/Worker labels
  - Service annotations
  - Pod annotations（Prometheus）
- **验证点**：
  - Labels 正确应用
  - Annotations 正确应用
  - 可以被监控系统识别

#### 6.4 ConfigMap 更新
- **目标**：验证配置更新
- **测试场景**：
  - 更新配置文件
  - 执行 update 命令
- **验证点**：
  - ConfigMap 更新
  - Pod 重启应用新配置

### 模块 7: 更新和升级测试 (Updates & Upgrades)

#### 7.1 镜像更新
- **目标**：验证镜像升级
- **测试场景**：
  - 更新 Master 镜像
  - 更新 Worker 镜像
  - 同时更新两者
- **验证点**：
  - 滚动更新成功
  - 服务不中断
  - 新镜像正常运行

#### 7.2 配置更新
- **目标**：验证配置更新
- **测试场景**：
  - 更新配置文件
  - 执行 update 命令
- **验证点**：
  - 配置正确更新
  - Pod 应用新配置

#### 7.3 滚动更新验证
- **目标**：验证滚动更新策略
- **验证点**：
  - 更新过程中服务可用
  - 旧 Pod 正常终止
  - 新 Pod 正常启动

### 模块 8: 调度策略测试 (Scheduling)

#### 8.1 Node Selector
- **目标**：验证节点选择
- **参数**：
  - `-Dkubernetes.master.node-selector=node-type=master`
  - `-Dkubernetes.worker.node-selector=node-type=worker`
- **验证点**：
  - Pod 调度到正确节点
  - Node selector 正确应用

#### 8.2 Affinity/Anti-affinity
- **目标**：验证亲和性配置
- **注意**：需要多节点环境
- **验证点**：
  - Pod 分布符合预期
  - 反亲和性生效

#### 8.3 Tolerations
- **目标**：验证容忍度配置
- **参数**：通过 Pod template
- **验证点**：
  - Tolerations 正确应用
  - Pod 可以调度到污点节点

#### 8.4 Priority Class
- **目标**：验证优先级
- **参数**：
  - `-Dkubernetes.pod.priority-class=high-priority`
- **验证点**：
  - Priority class 正确应用
  - 调度优先级正确

### 模块 9: 安全配置测试 (Security)

#### 9.1 Service Account
- **目标**：验证服务账户
- **参数**：
  - `-Dkubernetes.master.service-account=curvine-master`
  - `-Dkubernetes.worker.service-account=curvine-worker`
- **验证点**：
  - Service account 正确应用
  - RBAC 权限正确

#### 9.2 Image Pull Secrets
- **目标**：验证镜像拉取密钥
- **参数**：image_pull_secrets
- **验证点**：
  - Secrets 正确引用
  - 私有镜像可以拉取

#### 9.3 Security Context
- **目标**：验证安全上下文
- **参数**：通过 Pod template
- **验证点**：
  - Security context 正确应用
  - 权限限制正确

### 模块 10: 边界和异常测试 (Edge Cases)

#### 10.1 无效配置验证
- **目标**：验证配置验证逻辑
- **测试场景**：
  - 偶数 Master 副本（应该失败）
  - 无效集群 ID（应该失败）
  - 无效 Service 类型（应该失败）
  - 无效资源格式（应该失败）
- **验证点**：
  - 错误被正确捕获
  - 错误信息清晰

#### 10.2 资源冲突
- **目标**：验证资源冲突处理
- **测试场景**：
  - 重复部署同一集群（应该失败）
  - 资源名称冲突
- **验证点**：
  - 冲突被正确检测
  - 错误信息清晰

#### 10.3 并发操作
- **目标**：验证并发操作
- **测试场景**：
  - 同时执行多个 update
  - 在更新时执行 delete
- **验证点**：
  - 操作正确处理
  - 无数据损坏

#### 10.4 命名空间隔离
- **目标**：验证命名空间隔离
- **测试场景**：
  - 在不同命名空间部署相同集群 ID
  - 验证资源隔离
- **验证点**：
  - 资源正确隔离
  - 无冲突

## 测试执行策略

### 测试组织方式

**方案 A：模块化测试脚本**
- 每个模块一个测试脚本
- 可以独立运行
- 便于维护和调试

**方案 B：统一测试脚本**
- 一个主测试脚本
- 按模块组织测试函数
- 可以运行全部或选择模块

**推荐**：方案 B，但支持模块选择

### 测试执行流程

1. **环境准备**
   - 检查 minikube 状态
   - 检查镜像
   - 清理旧资源

2. **测试执行**
   - 按模块顺序执行
   - 每个测试独立验证
   - 记录测试结果

3. **结果报告**
   - 测试通过/失败统计
   - 详细错误信息
   - 配置验证结果

4. **清理**
   - 删除测试资源
   - 恢复环境

## 测试覆盖矩阵

| 功能模块 | 测试场景数 | 优先级 | 状态 |
|---------|-----------|--------|------|
| 基础部署 | 3 | P0 | 待实现 |
| 资源管理 | 3 | P0 | 待实现 |
| 高可用性 | 3 | P1 | 待实现 |
| 存储配置 | 3 | P0 | 待实现 |
| 网络配置 | 3 | P0 | 待实现 |
| 配置管理 | 4 | P0 | 待实现 |
| 更新升级 | 3 | P1 | 待实现 |
| 调度策略 | 4 | P2 | 待实现 |
| 安全配置 | 3 | P2 | 待实现 |
| 边界异常 | 4 | P3 | 待实现 |

**总计**: 33 个测试场景

## 环境要求

### 最小环境（当前）
- minikube（单节点）
- 可以测试：基础部署、资源管理、存储、网络（ClusterIP/NodePort）、配置管理、更新

### 理想环境（多节点）
- Kubernetes 集群（多节点）
- 可以测试：Node selector、Affinity、节点故障

### 生产环境
- 生产级 Kubernetes 集群
- 可以测试：所有场景，包括性能测试

## 测试数据

### 测试集群配置
- **开发环境**：Master 1, Worker 1
- **测试环境**：Master 3, Worker 3
- **生产环境**：Master 5, Worker 10+

### 测试镜像
- `curvine:latest`（当前）
- `curvine:v1.0.0`（用于升级测试）

## 成功标准

1. **功能正确性**：所有功能按预期工作
2. **配置准确性**：所有配置正确应用
3. **资源正确性**：所有 K8s 资源正确创建
4. **稳定性**：长时间运行无异常
5. **可恢复性**：故障后能自动恢复

## 下一步行动

1. ✅ 完成测试方案规划
2. ⏳ 实现测试脚本框架
3. ⏳ 实现各模块测试
4. ⏳ 执行测试并修复问题
5. ⏳ 生成测试报告

