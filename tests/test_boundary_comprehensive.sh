#!/bin/bash
# Comprehensive boundary tests for curvine-kube
# Tests all scenarios including edge cases

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }
log_test() { echo -e "${YELLOW}[TEST]${NC} $1"; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BINARY_PATH="$PROJECT_ROOT/target/release/curvine-kube"
TEMPLATE_DIR="$SCRIPT_DIR/pod-templates"
CONFIG_DIR=$(mktemp -d -t curvine-boundary-test-XXXXXX)
CONFIG_FILE="$CONFIG_DIR/curvine-cluster.toml"
CLUSTER_ID="boundary-test"
NAMESPACE="default"
TEST_FAILED=0

cleanup() {
    log_info "Cleaning up test resources..."
    if [ -f "$BINARY_PATH" ]; then
        $BINARY_PATH delete "$CLUSTER_ID" -n "$NAMESPACE" --delete-pvcs 2>/dev/null || true
    fi
    kubectl delete statefulset,deployment,svc,configmap,pvc,pod -n "$NAMESPACE" -l app="$CLUSTER_ID" --ignore-not-found=true --force --grace-period=0 2>/dev/null || true
    rm -rf "$CONFIG_DIR" 2>/dev/null || true
}
trap cleanup EXIT

# Create test configuration
create_config() {
    log_test "Creating test configuration..."
    cat > "$CONFIG_FILE" << 'EOF'
format_master = false
format_worker = false

[master]
meta_dir = "testing/meta"
rpc_port = 8995
web_port = 9000
log = { level = "info", log_dir = "stdout", file_name = "master.log" }

[journal]
journal_addrs = []
journal_dir = "testing/journal"
rpc_port = 8996

[worker]
dir_reserved = "0"
data_dir = ["[MEM:100MB]/data/mem"]
rpc_port = 8997
web_port = 9001
log = { level = "info", log_dir = "stdout", file_name = "worker.log" }
enable_s3_gateway = false

[client]
master_addrs = []
block_size_str = "64MB"
write_type = "cache_through"
read_type = "cache"

[fuse]

[log]
level = "info"
log_dir = "stdout"
file_name = "curvine.log"

[s3_gateway]
listen = "0.0.0.0:9900"
region = "us-east-1"
EOF
    log_success "Configuration created"
}

# Wait for pod
wait_for_pod() {
    local pod_name=$1
    local namespace=$2
    local timeout=${3:-180}
    local start_time=$(date +%s)
    local max_wait_for_creation=60
    local creation_elapsed=0
    local pod_exists=false
    
    log_info "Waiting for pod $pod_name (timeout: ${timeout}s)..."
    
    while [ $creation_elapsed -lt $max_wait_for_creation ]; do
        if kubectl get pod -n "$namespace" "$pod_name" &>/dev/null; then
            pod_exists=true
            break
        fi
        sleep 2
        creation_elapsed=$(($(date +%s) - start_time))
    done
    
    if [ "$pod_exists" = "false" ]; then
        log_error "Pod $pod_name was not created within ${max_wait_for_creation}s"
        return 1
    fi
    
    while true; do
        local elapsed=$(($(date +%s) - start_time))
        if [ $elapsed -gt $timeout ]; then
            log_error "Timeout waiting for pod $pod_name to be ready (waited ${elapsed}s)"
            return 1
        fi
        
        if ! kubectl get pod -n "$namespace" "$pod_name" &>/dev/null; then
            log_error "Pod $pod_name no longer exists"
            return 1
        fi
        
        local status=$(kubectl get pod -n "$namespace" "$pod_name" -o jsonpath='{.status.phase}' 2>/dev/null || echo "Unknown")
        local ready=$(kubectl get pod -n "$namespace" "$pod_name" -o jsonpath='{.status.containerStatuses[0].ready}' 2>/dev/null || echo "false")
        local restart_count=$(kubectl get pod -n "$namespace" "$pod_name" -o jsonpath='{.status.containerStatuses[0].restartCount}' 2>/dev/null || echo "0")
        
        if [ "$status" = "Failed" ] || [ "$status" = "Error" ]; then
            log_error "Pod $pod_name is in $status state"
            return 1
        fi
        
        if [ "$status" = "Running" ] && [ "$restart_count" -gt 3 ]; then
            log_error "Pod $pod_name is restarting too many times (restartCount: $restart_count)"
            return 1
        fi
        
        if [ "$status" = "Running" ] && [ "$ready" = "true" ]; then
            log_success "Pod $pod_name is ready (took ${elapsed}s)"
            return 0
        fi
        
        sleep 3
    done
}

# Test 1: Basic deploy
test_deploy() {
    log_test "Test 1: Deploy basic cluster"
    local output=$(timeout 60 $BINARY_PATH deploy -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --master-replicas 1 \
        --worker-replicas 1 \
        --image curvine:latest \
        --config-file "$CONFIG_FILE" 2>&1)
    local exit_code=$?
    
    if [ $exit_code -ne 0 ]; then
        log_error "Deploy failed"
        echo "$output"
        TEST_FAILED=1
        return 1
    fi
    
    if ! wait_for_pod "${CLUSTER_ID}-master-0" "$NAMESPACE" 180; then
        TEST_FAILED=1
        return 1
    fi
    
    if ! wait_for_pod "${CLUSTER_ID}-worker-0" "$NAMESPACE" 180; then
        TEST_FAILED=1
        return 1
    fi
    
    log_success "Test 1 passed: Basic deployment successful"
    return 0
}

# Test 2: Update with hostPath
test_update_hostpath() {
    log_test "Test 2: Update with hostPath pod-template"
    mkdir -p /tmp/curvine-master-extra /tmp/curvine-worker-extra 2>/dev/null || \
        sudo mkdir -p /tmp/curvine-master-extra /tmp/curvine-worker-extra
    chmod 777 /tmp/curvine-master-extra /tmp/curvine-worker-extra 2>/dev/null || \
        sudo chmod 777 /tmp/curvine-master-extra /tmp/curvine-worker-extra
    
    local output=$(timeout 60 $BINARY_PATH update -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --master-pod-template "$TEMPLATE_DIR/master-hostpath.yaml" \
        --worker-pod-template "$TEMPLATE_DIR/worker-hostpath.yaml" \
        --config-file "$CONFIG_FILE" 2>&1)
    local exit_code=$?
    
    if [ $exit_code -ne 0 ]; then
        log_error "Update with hostPath failed"
        echo "$output"
        TEST_FAILED=1
        return 1
    fi
    
    sleep 5
    # Verify master hostPath volume
    local master_volumes=$(kubectl get statefulset -n "$NAMESPACE" "${CLUSTER_ID}-master" -o jsonpath='{.spec.template.spec.volumes[*].name}' 2>/dev/null)
    if ! echo "$master_volumes" | grep -q "hostpath-extra"; then
        log_error "Test 2 failed: hostPath volume not found in master StatefulSet"
        TEST_FAILED=1
        return 1
    fi
    
    # Verify worker hostPath volume
    local worker_volumes=$(kubectl get statefulset -n "$NAMESPACE" "${CLUSTER_ID}-worker" -o jsonpath='{.spec.template.spec.volumes[*].name}' 2>/dev/null)
    if ! echo "$worker_volumes" | grep -q "hostpath-extra"; then
        log_error "Test 2 failed: hostPath volume not found in worker StatefulSet"
        TEST_FAILED=1
        return 1
    fi
    
    # Verify master volumeMount
    local master_mounts=$(kubectl get statefulset -n "$NAMESPACE" "${CLUSTER_ID}-master" -o jsonpath='{.spec.template.spec.containers[0].volumeMounts[*].name}' 2>/dev/null)
    if ! echo "$master_mounts" | grep -q "hostpath-extra"; then
        log_error "Test 2 failed: hostPath volumeMount not found in master container"
        TEST_FAILED=1
        return 1
    fi
    
    # Verify worker volumeMount
    local worker_mounts=$(kubectl get statefulset -n "$NAMESPACE" "${CLUSTER_ID}-worker" -o jsonpath='{.spec.template.spec.containers[0].volumeMounts[*].name}' 2>/dev/null)
    if ! echo "$worker_mounts" | grep -q "hostpath-extra"; then
        log_error "Test 2 failed: hostPath volumeMount not found in worker container"
        TEST_FAILED=1
        return 1
    fi
    
    log_success "Test 2 passed: hostPath volumes and mounts found in both master and worker"
    return 0
}

# Test 3: Invalid mountPath (should fail)
test_invalid_mountpath() {
    log_test "Test 3: Invalid mountPath validation (should fail)"
    local output
    output=$($BINARY_PATH update -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --master-pod-template "$TEMPLATE_DIR/master-invalid-mountpath.yaml" \
        --worker-pod-template "$TEMPLATE_DIR/worker-invalid-mountpath.yaml" \
        --config-file "$CONFIG_FILE" 2>&1)
    local exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        log_error "Test 3 failed: Should have rejected invalid mountPath"
        echo "Output: $output"
        TEST_FAILED=1
        return 1
    fi
    
    if echo "$output" | grep -qi "mount.*path.*mismatch\|Volume mount path mismatch"; then
        log_success "Test 3 passed: Invalid mountPath correctly rejected"
        return 0
    else
        log_error "Test 3 failed: Error message doesn't mention mountPath"
        echo "Output: $output"
        TEST_FAILED=1
        return 1
    fi
}

# Test 4: Wrong container name (should fail)
test_wrong_container_name() {
    log_test "Test 4: Wrong container name validation (should fail)"
    local output
    output=$($BINARY_PATH update -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --master-pod-template "$TEMPLATE_DIR/master-wrong-container-name.yaml" \
        --config-file "$CONFIG_FILE" 2>&1)
    local exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        log_error "Test 4 failed: Should have rejected wrong container name"
        echo "Output: $output"
        TEST_FAILED=1
        return 1
    fi
    
    if echo "$output" | grep -qi "container.*name.*mismatch\|Container name mismatch"; then
        log_success "Test 4 passed: Wrong container name correctly rejected"
        return 0
    else
        log_error "Test 4 failed: Error message doesn't mention container name"
        echo "Output: $output"
        TEST_FAILED=1
        return 1
    fi
}

# Test 5: Correct mountPath (should succeed)
test_correct_mountpath() {
    log_test "Test 5: Correct mountPath (should succeed)"
    mkdir -p /tmp/curvine-meta /tmp/curvine-journal /tmp/curvine-data 2>/dev/null || \
        sudo mkdir -p /tmp/curvine-meta /tmp/curvine-journal /tmp/curvine-data
    chmod 777 /tmp/curvine-meta /tmp/curvine-journal /tmp/curvine-data 2>/dev/null || \
        sudo chmod 777 /tmp/curvine-meta /tmp/curvine-journal /tmp/curvine-data
    
    local output=$(timeout 60 $BINARY_PATH update -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --master-pod-template "$TEMPLATE_DIR/master-correct-mountpath.yaml" \
        --worker-pod-template "$TEMPLATE_DIR/worker-correct-mountpath.yaml" \
        --config-file "$CONFIG_FILE" 2>&1)
    local exit_code=$?
    
    if [ $exit_code -ne 0 ]; then
        log_error "Test 5 failed: Update with correct mountPath failed"
        echo "$output"
        TEST_FAILED=1
        return 1
    fi
    
    log_success "Test 5 passed: Correct mountPath update successful"
    return 0
}

# Test 6: Update with tolerations
test_update_tolerations() {
    log_test "Test 6: Update with tolerations"
    local output=$(timeout 60 $BINARY_PATH update -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --master-pod-template "$TEMPLATE_DIR/master-tolerations.yaml" \
        --worker-pod-template "$TEMPLATE_DIR/worker-tolerations.yaml" \
        --config-file "$CONFIG_FILE" 2>&1)
    local exit_code=$?
    
    if [ $exit_code -ne 0 ]; then
        log_error "Test 6 failed: Update with tolerations failed"
        echo "$output"
        TEST_FAILED=1
        return 1
    fi
    
    sleep 5
    local tolerations=$(kubectl get statefulset -n "$NAMESPACE" "${CLUSTER_ID}-master" -o jsonpath='{.spec.template.spec.tolerations}' 2>/dev/null)
    if [ -n "$tolerations" ] && [ "$tolerations" != "null" ]; then
        log_success "Test 6 passed: Tolerations applied"
    else
        log_error "Test 6 failed: Tolerations not found"
        TEST_FAILED=1
        return 1
    fi
    return 0
}

# Test 7: Remove pod templates
test_remove_templates() {
    log_test "Test 7: Remove pod templates"
    local output=$(timeout 60 $BINARY_PATH update -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --config-file "$CONFIG_FILE" 2>&1)
    local exit_code=$?
    
    if [ $exit_code -ne 0 ]; then
        log_error "Test 7 failed: Remove templates failed"
        echo "$output"
        TEST_FAILED=1
        return 1
    fi
    
    sleep 5
    local volumes=$(kubectl get statefulset -n "$NAMESPACE" "${CLUSTER_ID}-master" -o jsonpath='{.spec.template.spec.volumes[*].name}' 2>/dev/null)
    if ! echo "$volumes" | grep -q "hostpath-extra"; then
        log_success "Test 7 passed: Templates removed"
    else
        log_error "Test 7 failed: Templates still present"
        TEST_FAILED=1
        return 1
    fi
    return 0
}

# Test 8: Update worker replicas
test_update_worker_replicas() {
    log_test "Test 8: Update worker replicas to 2"
    local output=$(timeout 60 $BINARY_PATH update -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --worker-replicas 2 \
        --config-file "$CONFIG_FILE" 2>&1)
    local exit_code=$?
    
    if [ $exit_code -ne 0 ]; then
        log_error "Test 8 failed: Update worker replicas failed"
        echo "$output"
        TEST_FAILED=1
        return 1
    fi
    
    sleep 10
    local replicas=$(kubectl get statefulset -n "$NAMESPACE" "${CLUSTER_ID}-worker" -o jsonpath='{.spec.replicas}' 2>/dev/null)
    if [ "$replicas" = "2" ]; then
        log_success "Test 8 passed: Worker replicas updated to 2"
    else
        log_error "Test 8 failed: Worker replicas not updated (expected: 2, got: $replicas)"
        TEST_FAILED=1
        return 1
    fi
    return 0
}

# Test 9: Update worker replicas back to 1
test_update_worker_replicas_back() {
    log_test "Test 9: Update worker replicas back to 1"
    local output=$(timeout 60 $BINARY_PATH update -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --worker-replicas 1 \
        --config-file "$CONFIG_FILE" 2>&1)
    local exit_code=$?
    
    if [ $exit_code -ne 0 ]; then
        log_error "Test 9 failed: Update worker replicas back failed"
        echo "$output"
        TEST_FAILED=1
        return 1
    fi
    
    sleep 10
    local replicas=$(kubectl get statefulset -n "$NAMESPACE" "${CLUSTER_ID}-worker" -o jsonpath='{.spec.replicas}' 2>/dev/null)
    if [ "$replicas" = "1" ]; then
        log_success "Test 9 passed: Worker replicas updated back to 1"
    else
        log_error "Test 9 failed: Worker replicas not updated (expected: 1, got: $replicas)"
        TEST_FAILED=1
        return 1
    fi
    return 0
}

# Test 10: Status check
test_status() {
    log_test "Test 10: Status check"
    local output=$(timeout 30 $BINARY_PATH status "$CLUSTER_ID" -n "$NAMESPACE" 2>&1)
    local exit_code=$?
    
    if [ $exit_code -ne 0 ]; then
        log_error "Test 10 failed: Status check failed"
        echo "$output"
        TEST_FAILED=1
        return 1
    fi
    
    if echo "$output" | grep -q "$CLUSTER_ID"; then
        log_success "Test 10 passed: Status check successful"
    else
        log_error "Test 10 failed: Status output doesn't contain cluster ID"
        TEST_FAILED=1
        return 1
    fi
    return 0
}

# Test 11: List clusters
test_list() {
    log_test "Test 11: List clusters"
    local output=$(timeout 30 $BINARY_PATH list -n "$NAMESPACE" 2>&1)
    local exit_code=$?
    
    if [ $exit_code -ne 0 ]; then
        log_error "Test 11 failed: List clusters failed"
        echo "$output"
        TEST_FAILED=1
        return 1
    fi
    
    if echo "$output" | grep -q "$CLUSTER_ID"; then
        log_success "Test 11 passed: List clusters successful"
    else
        log_error "Test 11 failed: Cluster not found in list"
        TEST_FAILED=1
        return 1
    fi
    return 0
}

# Test 12: Boundary - Non-existent template file
test_nonexistent_template() {
    log_test "Test 12: Boundary - Non-existent template file (should fail)"
    local output
    output=$($BINARY_PATH update -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --master-pod-template "/nonexistent/path/template.yaml" \
        --config-file "$CONFIG_FILE" 2>&1)
    local exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        log_error "Test 12 failed: Should have rejected non-existent file"
        echo "Output: $output"
        TEST_FAILED=1
        return 1
    fi
    
    if echo "$output" | grep -qi "not found\|does not exist\|cannot.*read"; then
        log_success "Test 12 passed: Non-existent file correctly rejected"
        return 0
    else
        log_error "Test 12 failed: Error message doesn't mention file not found"
        echo "Output: $output"
        TEST_FAILED=1
        return 1
    fi
}

# Test 13: Boundary - Invalid YAML
test_invalid_yaml() {
    log_test "Test 13: Boundary - Invalid YAML (should fail)"
    local invalid_template=$(mktemp)
    echo "invalid: yaml: [unclosed" > "$invalid_template"
    
    local output
    output=$($BINARY_PATH update -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --master-pod-template "$invalid_template" \
        --config-file "$CONFIG_FILE" 2>&1)
    local exit_code=$?
    
    rm -f "$invalid_template"
    
    if [ $exit_code -eq 0 ]; then
        log_error "Test 13 failed: Should have rejected invalid YAML"
        echo "Output: $output"
        TEST_FAILED=1
        return 1
    fi
    
    if echo "$output" | grep -qi "yaml\|parse.*error\|invalid.*format\|mapping values\|Failed to parse pod template"; then
        log_success "Test 13 passed: Invalid YAML correctly rejected"
        return 0
    else
        log_error "Test 13 failed: Error message doesn't mention YAML parsing"
        echo "Output: $output"
        TEST_FAILED=1
        return 1
    fi
}

# Test 14: Boundary - Update master replicas (should fail)
test_update_master_replicas() {
    log_test "Test 14: Boundary - Update master replicas (should fail)"
    local output
    output=$($BINARY_PATH update -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --master-replicas 3 \
        --config-file "$CONFIG_FILE" 2>&1)
    local exit_code=$?
    
    if [ $exit_code -eq 0 ]; then
        log_error "Test 14 failed: Should have rejected master replicas update"
        echo "Output: $output"
        TEST_FAILED=1
        return 1
    fi
    
    if echo "$output" | grep -qi "master.*replicas.*cannot.*update\|Master replicas cannot be updated"; then
        log_success "Test 14 passed: Master replicas update correctly rejected"
        return 0
    else
        log_error "Test 14 failed: Error message doesn't mention master replicas"
        echo "Output: $output"
        TEST_FAILED=1
        return 1
    fi
}

# Main execution
main() {
    echo ""
    echo "=========================================="
    echo "  Comprehensive Boundary Tests"
    echo "=========================================="
    echo ""
    echo "Cluster ID: $CLUSTER_ID"
    echo "Namespace: $NAMESPACE"
    echo "Config File: $CONFIG_FILE"
    echo ""
    
    # Pre-cleanup
    kubectl delete statefulset,deployment,svc,configmap,pvc,pod -n "$NAMESPACE" -l app="$CLUSTER_ID" --ignore-not-found=true --force --grace-period=0 >/dev/null 2>&1 || true
    sleep 2
    
    create_config
    echo ""
    
    test_deploy && echo "" || echo ""
    test_update_hostpath && echo "" || echo ""
    test_invalid_mountpath && echo "" || echo ""
    test_wrong_container_name && echo "" || echo ""
    test_correct_mountpath && echo "" || echo ""
    test_update_tolerations && echo "" || echo ""
    test_remove_templates && echo "" || echo ""
    test_update_worker_replicas && echo "" || echo ""
    test_update_worker_replicas_back && echo "" || echo ""
    test_status && echo "" || echo ""
    test_list && echo "" || echo ""
    test_nonexistent_template && echo "" || echo ""
    test_invalid_yaml && echo "" || echo ""
    test_update_master_replicas && echo "" || echo ""
    
    echo ""
    echo "=========================================="
    if [ $TEST_FAILED -eq 0 ]; then
        log_success "All boundary tests passed!"
        echo "=========================================="
        exit 0
    else
        log_error "Some boundary tests failed!"
        echo "=========================================="
        exit 1
    fi
}

main

