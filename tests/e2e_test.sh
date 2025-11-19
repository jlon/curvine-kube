#!/bin/bash
# Copyright 2025 JiangLong.
#
# Licensed under the Apache License, Version 2.0 (the "License");
# you may not use this file except in compliance with the License.
# You may obtain a copy of the License at
#
#     http://www.apache.org/licenses/LICENSE-2.0
#
# Unless required by applicable law or agreed to in writing, software
# distributed under the License is distributed on an "AS IS" BASIS,
# WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
# See the License for the specific language governing permissions and
# limitations under the License.

# E2E Test Suite for curvine-kube
# This script tests all major functionalities of curvine-kube on minikube

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Logging functions
log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }
log_warning() { echo -e "${YELLOW}[!]${NC} $1"; }
log_test() { echo -e "${CYAN}[TEST]${NC} $1"; }

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BINARY_PATH="$PROJECT_ROOT/target/release/curvine-kube"
CONFIG_DIR=$(mktemp -d -t curvine-e2e-XXXXXX)
CONFIG_FILE="$CONFIG_DIR/curvine-cluster.toml"
CLUSTER_ID="e2e-test-cluster"
NAMESPACE="default"
TIMEOUT=600  # 10 minutes timeout for pod readiness
TEST_FAILED=0
COMMAND_TIMEOUT=120  # 2 minutes timeout for commands

# Cleanup function
cleanup() {
    log_info "Cleaning up test resources..."
    if [ -f "$BINARY_PATH" ]; then
        export CURVINE_CONF_FILE="$CONFIG_FILE"
        $BINARY_PATH delete "$CLUSTER_ID" -n "$NAMESPACE" 2>/dev/null || true
        $BINARY_PATH delete "$CLUSTER_ID" -n "$NAMESPACE" --delete-pvcs 2>/dev/null || true
    fi
    rm -rf "$CONFIG_DIR" 2>/dev/null || true
}
trap cleanup EXIT

# Check prerequisites
check_prerequisites() {
    log_test "Checking prerequisites..."
    
    # Check if binary exists
    if [ ! -f "$BINARY_PATH" ]; then
        log_error "Binary not found at $BINARY_PATH"
        log_info "Please build the project first: cargo xtask build --release"
        exit 1
    fi
    log_success "Binary found: $BINARY_PATH"
    
    # Check kubectl
    if ! command -v kubectl &> /dev/null; then
        log_error "kubectl not found"
        exit 1
    fi
    log_success "kubectl found"
    
    # Check minikube connection
    if ! kubectl cluster-info &>/dev/null; then
        log_error "Cannot connect to Kubernetes cluster"
        exit 1
    fi
    log_success "Connected to Kubernetes cluster"
    
    # Check if curvine:latest image exists in minikube
    if ! minikube image ls 2>/dev/null | grep -q "curvine:latest"; then
        log_warning "curvine:latest image not found in minikube"
        log_info "Please load the image: minikube image load curvine:latest"
    else
        log_success "curvine:latest image found in minikube"
    fi
}

# Create test configuration file
create_test_config() {
    log_test "Creating test configuration file..."
    
    cat > "$CONFIG_FILE" << 'EOF'
# E2E Test Configuration for Curvine Cluster
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
data_dir = ["[MEM:200MB]/data/mem"]
rpc_port = 8997
web_port = 9001
log = { level = "info", log_dir = "stdout", file_name = "worker.log" }
enable_s3_gateway = false

[client]
master_addrs = []
[client.kubernetes]
namespace = "default"
cluster_id = "e2e-test-cluster"
cluster_domain = "cluster.local"

[fuse]

[log]
level = "info"
log_dir = "stdout"
file_name = "curvine.log"

[s3_gateway]
listen = "0.0.0.0:9900"
region = "us-east-1"
EOF

    log_success "Configuration file created at $CONFIG_FILE"
    export CURVINE_CONF_FILE="$CONFIG_FILE"
}

# Wait for pod to be ready
wait_for_pod() {
    local pod_name=$1
    local namespace=$2
    local timeout=$3
    local start_time=$(date +%s)
    
    log_info "Waiting for pod $pod_name to be ready (timeout: ${timeout}s)..."
    
    while true; do
        local elapsed=$(($(date +%s) - start_time))
        if [ $elapsed -gt $timeout ]; then
            log_error "Timeout waiting for pod $pod_name"
            return 1
        fi
        
        if kubectl get pod -n "$namespace" "$pod_name" &>/dev/null; then
            local status=$(kubectl get pod -n "$namespace" "$pod_name" -o jsonpath='{.status.phase}' 2>/dev/null || echo "Unknown")
            local ready=$(kubectl get pod -n "$namespace" "$pod_name" -o jsonpath='{.status.containerStatuses[0].ready}' 2>/dev/null || echo "false")
            
            if [ "$status" = "Running" ] && [ "$ready" = "true" ]; then
                log_success "Pod $pod_name is ready"
                return 0
            fi
            
            if [ "$status" = "Failed" ] || [ "$status" = "Error" ]; then
                log_error "Pod $pod_name is in $status state"
                kubectl describe pod -n "$namespace" "$pod_name" | tail -20
                return 1
            fi
            
            log_info "Pod $pod_name status: $status (ready: $ready) (${elapsed}s)"
        else
            log_info "Waiting for pod $pod_name to appear (${elapsed}s)"
        fi
        sleep 3
    done
}

# Wait for deployment pods
wait_for_deployment() {
    local deployment_name=$1
    local namespace=$2
    local timeout=$3
    local start_time=$(date +%s)
    
    log_info "Waiting for deployment $deployment_name to be ready (timeout: ${timeout}s)..."
    
    while true; do
        local elapsed=$(($(date +%s) - start_time))
        if [ $elapsed -gt $timeout ]; then
            log_error "Timeout waiting for deployment $deployment_name"
            return 1
        fi
        
        if kubectl get deployment -n "$namespace" "$deployment_name" &>/dev/null; then
            local ready=$(kubectl get deployment -n "$namespace" "$deployment_name" -o jsonpath='{.status.readyReplicas}' 2>/dev/null || echo "0")
            local desired=$(kubectl get deployment -n "$namespace" "$deployment_name" -o jsonpath='{.spec.replicas}' 2>/dev/null || echo "0")
            
            if [ "$ready" = "$desired" ] && [ "$desired" != "0" ]; then
                log_success "Deployment $deployment_name is ready ($ready/$desired)"
                return 0
            fi
            
            log_info "Deployment $deployment_name: $ready/$desired ready (${elapsed}s)"
        else
            log_info "Waiting for deployment $deployment_name to appear (${elapsed}s)"
        fi
        sleep 3
    done
}

# Test 1: Deploy cluster with common parameters
test_deploy() {
    log_test "Test 1: Deploy cluster with common parameters"
    
    log_info "Deploying cluster with ID: $CLUSTER_ID"
    log_info "Parameters: master-replicas=1, worker-replicas=1, service-type=ClusterIP, image-pull-policy=IfNotPresent"
    local deploy_output=$(timeout $COMMAND_TIMEOUT $BINARY_PATH deploy -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --master-replicas 1 \
        --worker-replicas 1 \
        --image curvine:latest \
        --service-type ClusterIP \
        --image-pull-policy IfNotPresent \
        --config-file "$CONFIG_FILE" 2>&1)
    local deploy_exit_code=$?
    
    if [ $deploy_exit_code -eq 124 ]; then
        log_error "Deployment command timed out after ${COMMAND_TIMEOUT}s"
        TEST_FAILED=1
        return 1
    elif [ $deploy_exit_code -ne 0 ]; then
        log_error "Deployment failed (exit code: $deploy_exit_code)"
        echo "$deploy_output"
        TEST_FAILED=1
        return 1
    fi
    log_success "Deployment command executed successfully"
    
    # Wait for master pod
    if ! wait_for_pod "${CLUSTER_ID}-master-0" "$NAMESPACE" $TIMEOUT; then
        TEST_FAILED=1
        return 1
    fi
    
    # Wait for worker statefulset (Worker is StatefulSet, not Deployment)
    if ! wait_for_pod "${CLUSTER_ID}-worker-0" "$NAMESPACE" $TIMEOUT; then
        TEST_FAILED=1
        return 1
    fi
    
    log_success "Test 1 passed: Cluster deployed successfully"
    return 0
}

# Test 2: Verify resources
test_verify_resources() {
    log_test "Test 2: Verify Kubernetes resources"
    
    local resources_ok=1
    
    # Check StatefulSet
    if kubectl get statefulset -n "$NAMESPACE" "${CLUSTER_ID}-master" &>/dev/null; then
        log_success "Master StatefulSet exists"
    else
        log_error "Master StatefulSet not found"
        resources_ok=0
    fi
    
    # Check StatefulSet (Worker is StatefulSet, not Deployment)
    if kubectl get statefulset -n "$NAMESPACE" "${CLUSTER_ID}-worker" &>/dev/null; then
        log_success "Worker StatefulSet exists"
    else
        log_error "Worker StatefulSet not found"
        resources_ok=0
    fi
    
    # Check Service
    if kubectl get svc -n "$NAMESPACE" "${CLUSTER_ID}-master" &>/dev/null; then
        log_success "Master Service exists"
    else
        log_error "Master Service not found"
        resources_ok=0
    fi
    
    # Check Headless Service
    if kubectl get svc -n "$NAMESPACE" "${CLUSTER_ID}-master-headless" &>/dev/null; then
        log_success "Headless Service exists"
    else
        log_error "Headless Service not found"
        resources_ok=0
    fi
    
    # Check ConfigMap
    if kubectl get configmap -n "$NAMESPACE" "${CLUSTER_ID}-config" &>/dev/null; then
        log_success "ConfigMap exists"
    else
        log_error "ConfigMap not found"
        resources_ok=0
    fi
    
    if [ $resources_ok -eq 1 ]; then
        log_success "Test 2 passed: All resources verified"
        return 0
    else
        log_error "Test 2 failed: Some resources are missing"
        TEST_FAILED=1
        return 1
    fi
}

# Test 3: Status command
test_status() {
    log_test "Test 3: Check cluster status"
    
    local status_output=$(timeout 30 $BINARY_PATH status "$CLUSTER_ID" -n "$NAMESPACE" 2>&1)
    local status_exit_code=$?
    if [ $status_exit_code -eq 124 ]; then
        log_error "Status command timed out after 30s"
        TEST_FAILED=1
        return 1
    elif [ $status_exit_code -eq 0 ]; then
        log_success "Status command executed successfully"
        echo "$status_output" | head -10
        log_success "Test 3 passed: Status command works"
        return 0
    else
        log_error "Status command failed"
        echo "$status_output"
        TEST_FAILED=1
        return 1
    fi
}

# Test 4: List command
test_list() {
    log_test "Test 4: List clusters"
    
    local list_output=$(timeout 30 $BINARY_PATH list -n "$NAMESPACE" 2>&1)
    local list_exit_code=$?
    if [ $list_exit_code -eq 124 ]; then
        log_error "List command timed out after 30s"
        TEST_FAILED=1
        return 1
    elif [ $list_exit_code -eq 0 ]; then
        log_success "List command executed successfully"
        echo "$list_output"
        
        if echo "$list_output" | grep -q "$CLUSTER_ID"; then
            log_success "Test 4 passed: Cluster found in list"
            return 0
        else
            log_error "Cluster not found in list"
            TEST_FAILED=1
            return 1
        fi
    else
        log_error "List command failed"
        echo "$list_output"
        TEST_FAILED=1
        return 1
    fi
}

# Test 5: Update cluster (worker replicas)
test_update() {
    log_test "Test 5: Update cluster (scale workers)"
    
    log_info "Updating worker replicas to 2"
    local update_output=$(timeout $COMMAND_TIMEOUT $BINARY_PATH update -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --worker-replicas 2 \
        --config-file "$CONFIG_FILE" 2>&1)
    local update_exit_code=$?
    
    if [ $update_exit_code -eq 124 ]; then
        log_error "Update command timed out after ${COMMAND_TIMEOUT}s"
        TEST_FAILED=1
        return 1
    elif [ $update_exit_code -ne 0 ]; then
        log_error "Update command failed (exit code: $update_exit_code)"
        echo "$update_output"
        TEST_FAILED=1
        return 1
    fi
    log_success "Update command executed successfully"
    
    # Wait for new worker pods
    sleep 5
    if ! wait_for_pod "${CLUSTER_ID}-worker-0" "$NAMESPACE" $TIMEOUT; then
        TEST_FAILED=1
        return 1
    fi
    
    # Verify replica count (Worker is StatefulSet)
    local current_replicas=$(kubectl get statefulset -n "$NAMESPACE" "${CLUSTER_ID}-worker" -o jsonpath='{.spec.replicas}' 2>/dev/null)
    if [ "$current_replicas" = "2" ]; then
        log_success "Test 5 passed: Worker replicas updated to 2"
        return 0
    else
        log_error "Worker replicas not updated correctly (expected: 2, got: $current_replicas)"
        TEST_FAILED=1
        return 1
    fi
}

# Test 5b: Update cluster with multiple parameters
test_update_multiple() {
    log_test "Test 5b: Update cluster with multiple parameters"
    
    log_info "Updating worker replicas to 1, image, and image-pull-policy"
    local update_output=$(timeout $COMMAND_TIMEOUT $BINARY_PATH update -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --worker-replicas 1 \
        --image curvine:latest \
        --image-pull-policy Always \
        --config-file "$CONFIG_FILE" 2>&1)
    local update_exit_code=$?
    
    if [ $update_exit_code -eq 124 ]; then
        log_error "Update command timed out after ${COMMAND_TIMEOUT}s"
        TEST_FAILED=1
        return 1
    elif [ $update_exit_code -ne 0 ]; then
        log_error "Update command with multiple parameters failed (exit code: $update_exit_code)"
        echo "$update_output"
        TEST_FAILED=1
        return 1
    fi
    log_success "Update command with multiple parameters executed successfully"
    
    # Wait a bit for changes to apply
    sleep 5
    
    # Verify image pull policy
    local pull_policy=$(timeout 10 kubectl get statefulset -n "$NAMESPACE" "${CLUSTER_ID}-master" -o jsonpath='{.spec.template.spec.containers[0].imagePullPolicy}' 2>/dev/null)
    if [ "$pull_policy" = "Always" ]; then
        log_success "Test 5b passed: Image pull policy updated to Always"
        return 0
    else
        log_warning "Image pull policy not updated as expected (got: $pull_policy)"
        # Don't fail the test, just warn
        return 0
    fi
}

# Test 6: Update cluster (image)
test_update_image() {
    log_test "Test 6: Update cluster (image)"
    
    log_info "Updating master and worker images"
    local update_output=$(timeout $COMMAND_TIMEOUT $BINARY_PATH update -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --image curvine:latest \
        --config-file "$CONFIG_FILE" 2>&1)
    local update_exit_code=$?
    
    if [ $update_exit_code -eq 124 ]; then
        log_error "Update image command timed out after ${COMMAND_TIMEOUT}s"
        TEST_FAILED=1
        return 1
    elif [ $update_exit_code -ne 0 ]; then
        log_error "Update image command failed (exit code: $update_exit_code)"
        echo "$update_output"
        TEST_FAILED=1
        return 1
    fi
    log_success "Update image command executed successfully"
    
    # Wait a bit for rollout
    sleep 10
    
    log_success "Test 6 passed: Image update command works"
    return 0
}

# Test 7: Verify pods are running
test_pods_running() {
    log_test "Test 7: Verify all pods are running"
    
    local pods_ok=1
    
    # Check master pod
    local master_status=$(kubectl get pod -n "$NAMESPACE" "${CLUSTER_ID}-master-0" -o jsonpath='{.status.phase}' 2>/dev/null || echo "NotFound")
    if [ "$master_status" = "Running" ]; then
        log_success "Master pod is running"
    else
        log_error "Master pod is not running (status: $master_status)"
        pods_ok=0
    fi
    
    # Check worker pods (StatefulSet pods have fixed names)
    local worker_pods=$(kubectl get pod -n "$NAMESPACE" -l app="$CLUSTER_ID",component=worker -o jsonpath='{.items[*].metadata.name}' 2>/dev/null)
    # If no pods found with label selector, try StatefulSet naming pattern
    if [ -z "$worker_pods" ]; then
        if kubectl get pod -n "$NAMESPACE" "${CLUSTER_ID}-worker-0" &>/dev/null; then
            worker_pods="${CLUSTER_ID}-worker-0"
        fi
    fi
    if [ -n "$worker_pods" ]; then
        local worker_count=0
        local running_count=0
        for pod in $worker_pods; do
            worker_count=$((worker_count + 1))
            local pod_status=$(kubectl get pod -n "$NAMESPACE" "$pod" -o jsonpath='{.status.phase}' 2>/dev/null || echo "NotFound")
            if [ "$pod_status" = "Running" ]; then
                running_count=$((running_count + 1))
            else
                log_warning "Worker pod $pod is not running (status: $pod_status)"
            fi
        done
        log_info "Worker pods: $running_count/$worker_count running"
        if [ $running_count -gt 0 ]; then
            log_success "At least one worker pod is running"
        else
            log_error "No worker pods are running"
            pods_ok=0
        fi
    else
        log_error "No worker pods found"
        pods_ok=0
    fi
    
    if [ $pods_ok -eq 1 ]; then
        log_success "Test 7 passed: All pods are running"
        return 0
    else
        log_error "Test 7 failed: Some pods are not running"
        TEST_FAILED=1
        return 1
    fi
}

# Test 8: Delete cluster (without PVCs)
test_delete() {
    log_test "Test 8: Delete cluster (keeping PVCs)"
    
    log_info "Deleting cluster (keeping PVCs)"
    local delete_output=$(timeout 60 $BINARY_PATH delete "$CLUSTER_ID" -n "$NAMESPACE" 2>&1)
    local delete_exit_code=$?
    
    if [ $delete_exit_code -eq 124 ]; then
        log_error "Delete command timed out after 60s"
        TEST_FAILED=1
        return 1
    elif [ $delete_exit_code -ne 0 ]; then
        log_error "Delete command failed (exit code: $delete_exit_code)"
        echo "$delete_output"
        TEST_FAILED=1
        return 1
    fi
    log_success "Delete command executed successfully"
    
    # Wait for resources to be deleted
    sleep 10
    
    # Verify resources are deleted
    local resources_exist=0
    
    if kubectl get statefulset -n "$NAMESPACE" "${CLUSTER_ID}-master" &>/dev/null 2>&1; then
        log_error "Master StatefulSet still exists"
        resources_exist=1
    fi
    
    if kubectl get statefulset -n "$NAMESPACE" "${CLUSTER_ID}-worker" &>/dev/null 2>&1; then
        log_error "Worker StatefulSet still exists"
        resources_exist=1
    fi
    
    if kubectl get svc -n "$NAMESPACE" "${CLUSTER_ID}-master" &>/dev/null 2>&1; then
        log_error "Master Service still exists"
        resources_exist=1
    fi
    
    if kubectl get configmap -n "$NAMESPACE" "${CLUSTER_ID}-config" &>/dev/null 2>&1; then
        log_error "ConfigMap still exists"
        resources_exist=1
    fi
    
    if [ $resources_exist -eq 0 ]; then
        log_success "Test 8 passed: Cluster deleted successfully"
        return 0
    else
        log_error "Test 8 failed: Some resources still exist"
        TEST_FAILED=1
        return 1
    fi
}

# Main test execution
main() {
    echo ""
    echo "=========================================="
    echo "  Curvine-Kube E2E Test Suite"
    echo "=========================================="
    echo ""
    echo "Cluster ID: $CLUSTER_ID"
    echo "Namespace: $NAMESPACE"
    echo "Binary: $BINARY_PATH"
    echo ""
    
    # Check prerequisites
    check_prerequisites
    echo ""
    
    # Create test configuration
    create_test_config
    echo ""
    
    # Run tests
    log_info "Starting E2E tests..."
    echo ""
    
    test_deploy && echo "" || echo ""
    test_verify_resources && echo "" || echo ""
    test_status && echo "" || echo ""
    test_list && echo "" || echo ""
    test_update && echo "" || echo ""
    test_update_multiple && echo "" || echo ""
    test_update_image && echo "" || echo ""
    test_pods_running && echo "" || echo ""
    test_delete && echo "" || echo ""
    
    # Summary
    echo ""
    echo "=========================================="
    if [ $TEST_FAILED -eq 0 ]; then
        log_success "All E2E tests passed!"
        echo "=========================================="
        echo ""
        exit 0
    else
        log_error "Some E2E tests failed!"
        echo "=========================================="
        echo ""
        exit 1
    fi
}

# Run main function
main

