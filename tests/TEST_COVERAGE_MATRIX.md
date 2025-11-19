# Curvine-Kube 测试覆盖矩阵

## 功能覆盖矩阵

| 功能类别 | 功能点 | 当前测试 | 生产环境测试 | 优先级 |
|---------|--------|---------|------------|--------|
| **部署功能** |
| 单副本部署 | ✅ | ✅ | P0 |
| 多副本部署 | ❌ | ✅ | P0 |
| 大规模部署 | ❌ | ✅ | P1 |
| Master 奇数验证 | ❌ | ✅ | P0 |
| **资源管理** |
| CPU 限制 | ❌ | ✅ | P0 |
| 内存限制 | ❌ | ✅ | P0 |
| 资源请求 | ❌ | ✅ | P0 |
| 资源配额 | ❌ | ✅ | P1 |
| **存储配置** |
| StorageClass | ❌ | ✅ | P0 |
| 存储大小 | ❌ | ✅ | P0 |
| PVC 管理 | ❌ | ✅ | P0 |
| **网络配置** |
| ClusterIP | ✅ | ✅ | P0 |
| NodePort | ❌ | ✅ | P0 |
| LoadBalancer | ❌ | ✅ | P1 |
| 端口验证 | ❌ | ✅ | P0 |
| **配置管理** |
| 动态参数 (-D) | ❌ | ✅ | P0 |
| 环境变量 | ❌ | ✅ | P0 |
| Labels | ❌ | ✅ | P0 |
| Annotations | ❌ | ✅ | P0 |
| ConfigMap 更新 | ❌ | ✅ | P0 |
| **更新升级** |
| 镜像更新 | ✅ | ✅ | P1 |
| 配置更新 | ❌ | ✅ | P1 |
| 滚动更新 | ❌ | ✅ | P1 |
| **高可用性** |
| Worker 扩缩容 | ✅ | ✅ | P1 |
| Pod 故障恢复 | ❌ | ✅ | P1 |
| 节点故障 | ❌ | ✅ | P2 |
| **调度策略** |
| Node Selector | ❌ | ✅ | P2 |
| Affinity | ❌ | ✅ | P2 |
| Tolerations | ❌ | ✅ | P2 |
| Priority Class | ❌ | ✅ | P2 |
| **安全配置** |
| Service Account | ❌ | ✅ | P2 |
| Image Pull Secrets | ❌ | ✅ | P2 |
| Security Context | ❌ | ✅ | P2 |
| **边界异常** |
| 无效配置验证 | ❌ | ✅ | P3 |
| 资源冲突 | ❌ | ✅ | P3 |
| 并发操作 | ❌ | ✅ | P3 |
| 命名空间隔离 | ❌ | ✅ | P3 |

## 参数覆盖矩阵

### Deploy 命令参数

| 参数 | 当前测试 | 生产测试 | 优先级 |
|------|---------|---------|--------|
| `--cluster-id` | ✅ | ✅ | P0 |
| `--namespace` | ✅ | ✅ | P0 |
| `--master-replicas` | ✅ (1) | ✅ (1,3,5) | P0 |
| `--worker-replicas` | ✅ (1) | ✅ (1,3,5,10) | P0 |
| `--master-image` | ✅ | ✅ | P0 |
| `--worker-image` | ✅ | ✅ | P0 |
| `--service-type` | ✅ (ClusterIP) | ✅ (所有类型) | P0 |
| `--image-pull-policy` | ✅ | ✅ | P0 |
| `--storage-class` | ❌ | ✅ | P0 |
| `--master-storage-class` | ❌ | ✅ | P0 |
| `--worker-storage-class` | ❌ | ✅ | P0 |
| `--master-storage-size` | ❌ | ✅ | P0 |
| `--worker-storage-size` | ❌ | ✅ | P0 |
| `--master-pod-template` | ❌ | ✅ | P2 |
| `--worker-pod-template` | ❌ | ✅ | P2 |
| `-D` (动态参数) | ❌ | ✅ | P0 |

### Update 命令参数

| 参数 | 当前测试 | 生产测试 | 优先级 |
|------|---------|---------|--------|
| `--worker-replicas` | ✅ | ✅ | P0 |
| `--master-image` | ✅ | ✅ | P1 |
| `--worker-image` | ✅ | ✅ | P1 |
| `--image-pull-policy` | ✅ | ✅ | P1 |
| `--service-type` | ❌ | ✅ | P1 |
| `--config-file` | ✅ | ✅ | P0 |
| `-D` (动态参数) | ❌ | ✅ | P0 |

### Delete 命令参数

| 参数 | 当前测试 | 生产测试 | 优先级 |
|------|---------|---------|--------|
| `--delete-pvcs` | ❌ | ✅ | P0 |

### 动态配置参数 (-D)

| 参数类别 | 参数示例 | 当前测试 | 生产测试 | 优先级 |
|---------|---------|---------|---------|--------|
| **资源** |
| CPU | `kubernetes.master.cpu=2.0` | ❌ | ✅ | P0 |
| Memory | `kubernetes.master.memory=4Gi` | ❌ | ✅ | P0 |
| **调度** |
| Node Selector | `kubernetes.master.node-selector=...` | ❌ | ✅ | P2 |
| Labels | `kubernetes.master.labels=...` | ❌ | ✅ | P0 |
| Annotations | `kubernetes.master.annotations=...` | ❌ | ✅ | P0 |
| **服务** |
| Service Type | `kubernetes.service.type=NodePort` | ❌ | ✅ | P0 |
| External IPs | `kubernetes.service.external-ips=...` | ❌ | ✅ | P1 |
| **环境变量** |
| Env Vars | `kubernetes.master.env.VAR=value` | ❌ | ✅ | P0 |
| **安全** |
| Service Account | `kubernetes.master.service-account=...` | ❌ | ✅ | P2 |
| **DNS/Priority** |
| DNS Policy | `kubernetes.pod.dns-policy=...` | ❌ | ✅ | P2 |
| Priority Class | `kubernetes.pod.priority-class=...` | ❌ | ✅ | P2 |

## 测试场景覆盖统计

### 按优先级统计

| 优先级 | 场景数 | 当前覆盖 | 覆盖率 |
|--------|--------|---------|--------|
| P0 (核心) | 18 | 3 | 16.7% |
| P1 (重要) | 6 | 2 | 33.3% |
| P2 (推荐) | 6 | 0 | 0% |
| P3 (完善) | 3 | 0 | 0% |
| **总计** | **33** | **5** | **15.2%** |

### 按模块统计

| 模块 | 场景数 | 当前覆盖 | 覆盖率 |
|------|--------|---------|--------|
| 基础部署 | 3 | 1 | 33.3% |
| 资源管理 | 3 | 0 | 0% |
| 高可用性 | 3 | 1 | 33.3% |
| 存储配置 | 3 | 0 | 0% |
| 网络配置 | 3 | 1 | 33.3% |
| 配置管理 | 4 | 0 | 0% |
| 更新升级 | 3 | 2 | 66.7% |
| 调度策略 | 4 | 0 | 0% |
| 安全配置 | 3 | 0 | 0% |
| 边界异常 | 4 | 0 | 0% |

## 关键差距分析

### 1. 核心功能缺失（P0）
- ❌ 多副本部署测试
- ❌ 资源限制测试
- ❌ 存储配置测试
- ❌ 动态参数测试
- ❌ NodePort 服务测试
- ❌ Labels/Annotations 测试

### 2. 高可用性测试不足（P1）
- ❌ Master 奇数副本验证
- ❌ Pod 故障恢复测试
- ❌ 配置更新测试

### 3. 生产特性未测试（P2）
- ❌ 调度策略测试
- ❌ 安全配置测试

### 4. 边界情况未覆盖（P3）
- ❌ 无效配置验证
- ❌ 资源冲突测试
- ❌ 并发操作测试

## 改进建议

1. **立即实施**（P0 优先级）
   - 实现多副本部署测试
   - 实现资源限制测试
   - 实现存储配置测试
   - 实现动态参数测试

2. **短期实施**（P1 优先级）
   - 实现高可用性测试
   - 实现配置更新测试

3. **中期实施**（P2 优先级）
   - 实现调度策略测试
   - 实现安全配置测试

4. **长期完善**（P3 优先级）
   - 实现边界异常测试

## 测试执行计划

### 阶段 1: 核心功能（P0）- 预计 2-3 天
- 多副本部署
- 资源管理
- 存储配置
- 网络配置（NodePort）
- 动态参数

### 阶段 2: 高可用性（P1）- 预计 1-2 天
- Master 奇数验证
- Pod 故障恢复
- 配置更新

### 阶段 3: 生产特性（P2）- 预计 1-2 天
- 调度策略
- 安全配置

### 阶段 4: 边界异常（P3）- 预计 1 天
- 无效配置
- 资源冲突
- 并发操作

**总预计时间**: 5-8 天

