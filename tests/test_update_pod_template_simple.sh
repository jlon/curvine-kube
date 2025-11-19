#!/bin/bash
# Simplified test to find bottlenecks

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
TEMPLATE_DIR="$SCRIPT_DIR/pod-templates"
CONFIG_DIR=$(mktemp -d -t curvine-test-XXXXXX)
CONFIG_FILE="$CONFIG_DIR/curvine-cluster.toml"
CLUSTER_ID="pod-template-test"
NAMESPACE="default"
STEP_TIMEOUT=60  # 60 seconds per step
POD_WAIT_TIMEOUT=180  # 3 minutes for pod startup (deploy waits up to 5min, but we check earlier)

cleanup() {
    log_info "Cleaning up all resources..."
    if [ -f "$BINARY_PATH" ]; then
        $BINARY_PATH delete "$CLUSTER_ID" -n "$NAMESPACE" --delete-pvcs 2>/dev/null || true
    fi
    # Force cleanup
    kubectl delete statefulset,deployment,svc,configmap,pvc -n "$NAMESPACE" -l app="$CLUSTER_ID" --ignore-not-found=true 2>/dev/null || true
    kubectl delete pod -n "$NAMESPACE" -l app="$CLUSTER_ID" --force --grace-period=0 2>/dev/null || true
    sleep 2
    rm -rf "$CONFIG_DIR" 2>/dev/null || true
}
trap cleanup EXIT

# Pre-cleanup before starting
pre_cleanup() {
    log_info "Pre-cleaning any existing resources..."
    kubectl delete statefulset,deployment,svc,configmap,pvc -n "$NAMESPACE" -l app="$CLUSTER_ID" --ignore-not-found=true 2>/dev/null || true
    sleep 2
}

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
cluster_id = "pod-template-test"
EOF
    export CURVINE_CONF_FILE="$CONFIG_FILE"
    log_success "Config created"
}

# Step 1: Deploy basic cluster
step1_deploy() {
    log_step "Step 1: Deploy basic cluster (timeout: ${STEP_TIMEOUT}s)"
    local start=$(date +%s)
    
    local output=$(timeout $STEP_TIMEOUT $BINARY_PATH deploy -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --master-replicas 1 \
        --worker-replicas 1 \
        --image curvine:latest \
        
        --config-file "$CONFIG_FILE" 2>&1)
    local exit_code=$?
    local elapsed=$(($(date +%s) - start))
    
    if [ $exit_code -eq 124 ]; then
        log_error "Step 1 TIMED OUT after ${STEP_TIMEOUT}s"
        return 1
    elif [ $exit_code -ne 0 ]; then
        log_error "Step 1 FAILED (exit: $exit_code, elapsed: ${elapsed}s)"
        echo "$output" | tail -20
        return 1
    fi
    
    log_success "Step 1 completed in ${elapsed}s"
    return 0
}

# Step 2: Wait for pods with detailed diagnostics
step2_wait_pods() {
    log_step "Step 2: Wait for pods to be ready (timeout: ${POD_WAIT_TIMEOUT}s)"
    local start=$(date +%s)
    local timeout=$POD_WAIT_TIMEOUT
    local check_interval=3
    local last_status_time=$start
    
    while [ $(($(date +%s) - start)) -lt $timeout ]; do
        local elapsed=$(($(date +%s) - start))
        local master_phase=$(kubectl get pod -n "$NAMESPACE" "${CLUSTER_ID}-master-0" -o jsonpath='{.status.phase}' 2>/dev/null || echo "NotFound")
        local worker_phase=$(kubectl get pod -n "$NAMESPACE" "${CLUSTER_ID}-worker-0" -o jsonpath='{.status.phase}' 2>/dev/null || echo "NotFound")
        
        # Print status every 10 seconds
        if [ $(($elapsed - $last_status_time)) -ge 10 ]; then
            log_info "Status check (${elapsed}s): Master=$master_phase, Worker=$worker_phase"
            last_status_time=$elapsed
            
            # If pod exists but not running, show details
            if [ "$master_phase" != "NotFound" ] && [ "$master_phase" != "Running" ]; then
                local master_reason=$(kubectl get pod -n "$NAMESPACE" "${CLUSTER_ID}-master-0" -o jsonpath='{.status.containerStatuses[0].state.waiting.reason}' 2>/dev/null || echo "")
                if [ -n "$master_reason" ]; then
                    log_info "Master waiting reason: $master_reason"
                fi
            fi
        fi
        
        if [ "$master_phase" = "Running" ] && [ "$worker_phase" = "Running" ]; then
            local master_ready=$(kubectl get pod -n "$NAMESPACE" "${CLUSTER_ID}-master-0" -o jsonpath='{.status.containerStatuses[0].ready}' 2>/dev/null || echo "false")
            local worker_ready=$(kubectl get pod -n "$NAMESPACE" "${CLUSTER_ID}-worker-0" -o jsonpath='{.status.containerStatuses[0].ready}' 2>/dev/null || echo "false")
            
            if [ "$master_ready" = "true" ] && [ "$worker_ready" = "true" ]; then
                log_success "Step 2 completed in ${elapsed}s - Pods are running and ready"
                return 0
            fi
        fi
        
        # Check for failure states immediately
        if [ "$master_phase" = "Failed" ] || [ "$master_phase" = "Error" ]; then
            log_error "Step 2 FAILED - Master pod in $master_phase state"
            kubectl describe pod -n "$NAMESPACE" "${CLUSTER_ID}-master-0" 2>&1 | tail -30
            kubectl logs -n "$NAMESPACE" "${CLUSTER_ID}-master-0" --tail=30 2>&1
            return 1
        fi
        
        if [ "$worker_phase" = "Failed" ] || [ "$worker_phase" = "Error" ]; then
            log_error "Step 2 FAILED - Worker pod in $worker_phase state"
            kubectl describe pod -n "$NAMESPACE" "${CLUSTER_ID}-worker-0" 2>&1 | tail -30
            kubectl logs -n "$NAMESPACE" "${CLUSTER_ID}-worker-0" --tail=30 2>&1
            return 1
        fi
        
        # Check for CrashLoopBackOff
        local master_restarts=$(kubectl get pod -n "$NAMESPACE" "${CLUSTER_ID}-master-0" -o jsonpath='{.status.containerStatuses[0].restartCount}' 2>/dev/null || echo "0")
        if [ "$master_restarts" -gt 3 ]; then
            log_error "Step 2 FAILED - Master pod restarted $master_restarts times (likely CrashLoopBackOff)"
            kubectl logs -n "$NAMESPACE" "${CLUSTER_ID}-master-0" --tail=50 2>&1
            return 1
        fi
        
        sleep $check_interval
    done
    
    log_error "Step 2 TIMED OUT - Pods not ready after ${timeout}s"
    log_info "Final status:"
    kubectl get pod -n "$NAMESPACE" -l app="$CLUSTER_ID" 2>&1
    log_info "Master pod events:"
    kubectl get events -n "$NAMESPACE" --field-selector involvedObject.name="${CLUSTER_ID}-master-0" --sort-by='.lastTimestamp' 2>&1 | tail -10
    return 1
}

# Step 3: Update with hostPath template
step3_update_hostpath() {
    log_step "Step 3: Update with hostPath template (timeout: ${STEP_TIMEOUT}s)"
    local start=$(date +%s)
    
    local master_template=$(cd "$TEMPLATE_DIR" && pwd)/master-hostpath.yaml
    local worker_template=$(cd "$TEMPLATE_DIR" && pwd)/worker-hostpath.yaml
    
    if [ ! -f "$master_template" ]; then
        log_error "Master template not found: $master_template"
        return 1
    fi
    
    log_info "Using templates: $master_template, $worker_template"
    
    # Create directories
    mkdir -p /tmp/curvine-master-extra /tmp/curvine-worker-extra 2>/dev/null || true
    chmod 777 /tmp/curvine-master-extra /tmp/curvine-worker-extra 2>/dev/null || true
    
    log_info "Executing update command (timeout: ${STEP_TIMEOUT}s)..."
    local output=$(timeout $STEP_TIMEOUT $BINARY_PATH update -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --master-pod-template "$master_template" \
        --worker-pod-template "$worker_template" \
        --config-file "$CONFIG_FILE" 2>&1)
    local exit_code=$?
    local elapsed=$(($(date +%s) - start))
    
    log_info "Update command output (first 15 lines):"
    echo "$output" | head -15
    
    if [ $exit_code -eq 124 ]; then
        log_error "Step 3 TIMED OUT after ${STEP_TIMEOUT}s"
        echo "$output" | tail -30
        return 1
    elif [ $exit_code -ne 0 ]; then
        log_error "Step 3 FAILED (exit: $exit_code, elapsed: ${elapsed}s)"
        echo "$output" | tail -30
        return 1
    fi
    
    log_success "Step 3 completed in ${elapsed}s"
    echo "$output" | grep -E "(applied|updated|success)" | head -5
    return 0
}

# Step 4: Verify update
step4_verify() {
    log_step "Step 4: Verify update (timeout: 60s)"
    local start=$(date +%s)
    
    sleep 5  # Wait for StatefulSet to update
    
    local master_volumes=$(timeout 10 kubectl get statefulset -n "$NAMESPACE" "${CLUSTER_ID}-master" -o jsonpath='{.spec.template.spec.volumes[*].name}' 2>/dev/null || echo "")
    log_info "Master volumes: $master_volumes"
    
    if echo "$master_volumes" | grep -q "hostpath-extra"; then
        log_success "Step 4 passed - hostPath volume found"
        return 0
    else
        log_error "Step 4 FAILED - hostPath volume not found"
        return 1
    fi
}

# Step 5: Test invalid mountPath (should fail)
step5_test_invalid() {
    log_step "Step 5: Test invalid mountPath validation (timeout: ${STEP_TIMEOUT}s)"
    local start=$(date +%s)
    
    local master_template=$(cd "$TEMPLATE_DIR" && pwd)/master-invalid-mountpath.yaml
    if [ ! -f "$master_template" ]; then
        log_error "Invalid template not found"
        return 1
    fi
    
    local output=$(timeout $STEP_TIMEOUT $BINARY_PATH update -c "$CLUSTER_ID" -n "$NAMESPACE" \
        --master-pod-template "$master_template" \
        --config-file "$CONFIG_FILE" 2>&1)
    local exit_code=$?
    local elapsed=$(($(date +%s) - start))
    
    # Check if the error is about mountPath validation (regardless of exit code)
    if echo "$output" | grep -qi "mount.*path.*mismatch\|mountPath.*mismatch\|Volume mount path"; then
        log_success "Step 5 passed in ${elapsed}s - Invalid mountPath correctly rejected"
        return 0
    fi
    
    if [ $exit_code -eq 124 ]; then
        log_error "Step 5 TIMED OUT after ${STEP_TIMEOUT}s"
        return 1
    elif [ $exit_code -eq 0 ]; then
        log_error "Step 5 FAILED - Should have rejected invalid mountPath but succeeded"
        echo "$output" | tail -10
        return 1
    else
        log_error "Step 5 FAILED - Update failed but error is not about mountPath validation"
        echo "$output" | tail -15
        return 1
    fi
}

# Main
main() {
    echo ""
    echo "=========================================="
    echo "  Pod-Template Test (Step-by-Step)"
    echo "=========================================="
    echo ""
    
    pre_cleanup
    echo ""
    
    create_config
    echo ""
    
    step1_deploy || { log_error "Stopped at Step 1"; exit 1; }
    echo ""
    
    step2_wait_pods || { log_error "Stopped at Step 2"; exit 1; }
    echo ""
    
    step3_update_hostpath || { log_error "Stopped at Step 3"; exit 1; }
    echo ""
    
    step4_verify || { log_error "Stopped at Step 4"; exit 1; }
    echo ""
    
    step5_test_invalid || { log_error "Stopped at Step 5"; exit 1; }
    echo ""
    
    log_success "All steps completed!"
    exit 0
}

main

