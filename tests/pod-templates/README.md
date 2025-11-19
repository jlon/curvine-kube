# Pod Template 示例文件

这些 pod-template 文件用于测试 curvine-kube 的 update 功能，特别是测试不同 pod-template 场景。

## 文件说明

### master-hostpath.yaml / worker-hostpath.yaml
**用途**: 测试 hostPath 存储场景

**特性**:
- 添加 hostPath volume (`/tmp/curvine-{master,worker}-data`)
- 设置 Security Context (runAsUser: 1000)
- 配置资源限制 (CPU/Memory)

**适用场景**:
- 开发环境快速测试
- 单节点集群（如 minikube）
- 需要直接访问节点文件系统

### master-tolerations.yaml / worker-tolerations.yaml
**用途**: 测试 tolerations 和资源限制

**特性**:
- 添加 tolerations（允许调度到 control-plane 节点）
- 配置资源请求和限制
- 适用于 minikube 单节点环境

**适用场景**:
- 单节点集群（minikube）
- 需要调度到 master 节点
- 资源限制测试

## 使用方法

### 在 deploy 时使用
```bash
curvine-kube deploy -c my-cluster \
  --master-pod-template tests/pod-templates/master-hostpath.yaml \
  --worker-pod-template tests/pod-templates/worker-hostpath.yaml \
  --config-file curvine-cluster.toml
```

### 在 update 时使用
```bash
curvine-kube update -c my-cluster \
  --master-pod-template tests/pod-templates/master-hostpath.yaml \
  --worker-pod-template tests/pod-templates/worker-hostpath.yaml \
  --config-file curvine-cluster.toml
```

### 使用动态配置 (-D)
```bash
curvine-kube deploy -c my-cluster \
  -Dkubernetes.master.pod-template=tests/pod-templates/master-hostpath.yaml \
  -Dkubernetes.worker.pod-template=tests/pod-templates/worker-hostpath.yaml \
  --config-file curvine-cluster.toml
```

## 注意事项

1. **hostPath 权限**: 确保 `/tmp/curvine-*-data` 目录有正确的权限
   ```bash
   sudo mkdir -p /tmp/curvine-master-data /tmp/curvine-worker-data
   sudo chmod 777 /tmp/curvine-master-data /tmp/curvine-worker-data
   ```

2. **路径**: pod-template 文件路径可以是绝对路径或相对路径（相对于当前工作目录）

3. **合并行为**: pod-template 会与 builder 生成的 Pod spec 合并：
   - Template 中的 volumes 会添加到 builder volumes
   - Template 中的 volumeMounts 会添加到 builder volumeMounts
   - Template 中的其他字段（如 securityContext, tolerations）会覆盖 builder 的默认值

4. **容器名称**: 必须确保 template 中的容器名称与 curvine-kube 使用的名称匹配：
   - Master: `master`
   - Worker: `worker`

## 测试

运行专门的 pod-template 测试：
```bash
./tests/test_update_pod_template.sh
```

这个测试会验证：
1. 基础部署（无 pod-template）
2. 使用 hostPath pod-template 更新
3. 使用 tolerations pod-template 更新
4. 移除 pod-template（回到默认）
5. 验证 pod-template 合并行为

