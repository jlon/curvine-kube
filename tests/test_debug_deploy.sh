#!/bin/bash
# Debug script to find where deploy command hangs

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

log_info() { echo -e "${BLUE}[INFO]${NC} $1"; }
log_success() { echo -e "${GREEN}[✓]${NC} $1"; }
log_error() { echo -e "${RED}[✗]${NC} $1"; }
log_step() { echo -e "${YELLOW}[STEP]${NC} $1"; }

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
BINARY_PATH="$PROJECT_ROOT/target/release/curvine-kube"
CONFIG_DIR=$(mktemp -d -t curvine-test-XXXXXX)
CONFIG_FILE="$CONFIG_DIR/curvine-cluster.toml"
CLUSTER_ID="debug-test"
NAMESPACE="default"

cleanup() {
    log_info "Cleaning up..."
    if [ -f "$BINARY_PATH" ]; then
        timeout 5 $BINARY_PATH delete "$CLUSTER_ID" -n "$NAMESPACE" --delete-pvcs 2>/dev/null || true
    fi
    kubectl delete statefulset,deployment,svc,configmap,pvc,pod -n "$NAMESPACE" -l app="$CLUSTER_ID" --ignore-not-found=true --force --grace-period=0 2>/dev/null || true
    rm -rf "$CONFIG_DIR" 2>/dev/null || true
}
trap cleanup EXIT

# Create minimal config
create_config() {
    log_step "Creating test configuration..."
    cat > "$CONFIG_FILE" << 'EOF'
format_master = false
format_worker = false

[master]
meta_dir = "testing/meta"
rpc_port = 8995
web_port = 9000
graceful_shutdown = false

[journal]
journal_dir = "testing/journal"
rpc_port = 8996

[worker]
data_dir = ["[MEM:100MB]/data/mem"]
rpc_port = 8997
graceful_shutdown = false

[client.kubernetes]
namespace = "default"
cluster_id = "debug-test"
EOF
    export CURVINE_CONF_FILE="$CONFIG_FILE"
    log_success "Config created"
}

# Test deploy with timeout
test_deploy() {
    log_step "Testing deploy command with 15s timeout..."
    local start=$(date +%s)
    
    # Run deploy in background and monitor
    ($BINARY_PATH deploy -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --master-replicas 1 \
        --worker-replicas 1 \
        --image curvine:latest \
        
        --config-file "$CONFIG_FILE" 2>&1) &
    local deploy_pid=$!
    
    # Monitor process
    local elapsed=0
    local timeout=15
    while kill -0 $deploy_pid 2>/dev/null && [ $elapsed -lt $timeout ]; do
        sleep 1
        elapsed=$(($(date +%s) - start))
        if [ $((elapsed % 3)) -eq 0 ]; then
            log_info "Deploy running... (${elapsed}s)"
            # Check if process is stuck
            local state=$(ps -o state= -p $deploy_pid 2>/dev/null || echo "gone")
            log_info "Process state: $state"
        fi
    done
    
    if kill -0 $deploy_pid 2>/dev/null; then
        log_error "Deploy TIMED OUT after ${timeout}s - killing process"
        kill -TERM $deploy_pid 2>/dev/null || true
        sleep 2
        kill -KILL $deploy_pid 2>/dev/null || true
        return 1
    else
        wait $deploy_pid
        local exit_code=$?
        local total_elapsed=$(($(date +%s) - start))
        if [ $exit_code -eq 0 ]; then
            log_success "Deploy completed in ${total_elapsed}s"
            return 0
        else
            log_error "Deploy failed (exit: $exit_code, elapsed: ${total_elapsed}s)"
            return 1
        fi
    fi
}

main() {
    echo ""
    echo "=========================================="
    echo "  Debug Deploy Command"
    echo "=========================================="
    echo ""
    
    # Cleanup first
    log_info "Pre-cleaning..."
    kubectl delete statefulset,deployment,svc,configmap,pvc,pod -n "$NAMESPACE" -l app="$CLUSTER_ID" --ignore-not-found=true --force --grace-period=0 2>&1 | head -3
    sleep 2
    echo ""
    
    create_config
    echo ""
    
    test_deploy
    echo ""
    
    log_info "Final cluster state:"
    kubectl get all,pvc -n "$NAMESPACE" -l app="$CLUSTER_ID" 2>&1 | head -10
}

main

