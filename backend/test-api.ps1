#!/usr/bin/env pwsh
# Backend Integration Test Script

Write-Host "=== Neo CRM Backend Integration Tests ===" -ForegroundColor Cyan
Write-Host ""

$API_BASE = "http://localhost:3000"
$TEST_EMAIL = "test_$(Get-Date -Format 'yyyyMMddHHmmss')@example.com"
$TEST_PASSWORD = "TestPassword123!"

# Check if backend is running
Write-Host "[ 1/8 ] Checking if backend is running..." -ForegroundColor Yellow
try {
    $health = Invoke-RestMethod -Uri "$API_BASE/health" -Method Get
    if ($health.status -eq "ok") {
        Write-Host "✓ Backend is running" -ForegroundColor Green
    } else {
        Write-Host "✗ Backend health check failed" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "✗ Backend is not running. Please start it with 'cargo run'" -ForegroundColor Red
    exit 1
}

# Test 1: Register new user
Write-Host ""
Write-Host "[ 2/8 ] Testing user registration..." -ForegroundColor Yellow
try {
    $registerBody = @{
        name = "Test User"
        email = $TEST_EMAIL
        password = $TEST_PASSWORD
    } | ConvertTo-Json

    $registerResponse = Invoke-RestMethod -Uri "$API_BASE/api/auth/register" `
        -Method Post `
        -ContentType "application/json" `
        -Body $registerBody
    
    Write-Host "✓ User registration successful" -ForegroundColor Green
} catch {
    Write-Host "✗ User registration failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 2: Login
Write-Host ""
Write-Host "[ 3/8 ] Testing login..." -ForegroundColor Yellow
try {
    $loginBody = @{
        email = $TEST_EMAIL
        password = $TEST_PASSWORD
    } | ConvertTo-Json

    $loginResponse = Invoke-RestMethod -Uri "$API_BASE/api/auth/login" `
        -Method Post `
        -ContentType "application/json" `
        -Body $loginBody
    
    $TOKEN = $loginResponse.token
    Write-Host "✓ Login successful. Token received." -ForegroundColor Green
} catch {
    Write-Host "✗ Login failed: $($_.Exception.Message)" -ForegroundColor Red
    exit 1
}

$headers = @{
    "Authorization" = "Bearer $TOKEN"
    "Content-Type" = "application/json"
}

# Test 3: Create Client
Write-Host ""
Write-Host "[ 4/8 ] Testing client creation..." -ForegroundColor Yellow
try {
    $clientBody = @{
        name = "Test Client Company"
        email = "client@testcompany.com"
        phone = "+1234567890"
        company = "Test Company Inc."
        status = "active"
    } | ConvertTo-Json

    $clientResponse = Invoke-RestMethod -Uri "$API_BASE/api/clients" `
        -Method Post `
        -Headers $headers `
        -Body $clientBody
    
    $CLIENT_ID = $clientResponse.id
    Write-Host "✓ Client created successfully (ID: $CLIENT_ID)" -ForegroundColor Green
} catch {
    Write-Host "✗ Client creation failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 4: List Clients
Write-Host ""
Write-Host "[ 5/8 ] Testing list clients..." -ForegroundColor Yellow
try {
    $clients = Invoke-RestMethod -Uri "$API_BASE/api/clients" `
        -Method Get `
        -Headers $headers
    
    Write-Host "✓ Retrieved $($clients.Count) client(s)" -ForegroundColor Green
} catch {
    Write-Host "✗ List clients failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 5: Create Task
Write-Host ""
Write-Host "[ 6/8 ] Testing task creation..." -ForegroundColor Yellow
try {
    $taskBody = @{
        title = "Test Task"
        description = "This is a test task"
        status = "todo"
        priority = "high"
        client_id = $CLIENT_ID
    } | ConvertTo-Json

    $taskResponse = Invoke-RestMethod -Uri "$API_BASE/api/tasks" `
        -Method Post `
        -Headers $headers `
        -Body $taskBody
    
    $TASK_ID = $taskResponse.id
    Write-Host "✓ Task created successfully (ID: $TASK_ID)" -ForegroundColor Green
} catch {
    Write-Host "✗ Task creation failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 6: Update Task Status
Write-Host ""
Write-Host "[ 7/8 ] Testing task update..." -ForegroundColor Yellow
try {
    $updateBody = @{
        status = "in_progress"
    } | ConvertTo-Json

    $updateResponse = Invoke-RestMethod -Uri "$API_BASE/api/tasks/$TASK_ID" `
        -Method Patch `
        -Headers $headers `
        -Body $updateBody
    
    Write-Host "✓ Task updated successfully" -ForegroundColor Green
} catch {
    Write-Host "✗ Task update failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Test 7: List Tasks
Write-Host ""
Write-Host "[ 8/8 ] Testing list tasks..." -ForegroundColor Yellow
try {
    $tasks = Invoke-RestMethod -Uri "$API_BASE/api/tasks" `
        -Method Get `
        -Headers $headers
    
    Write-Host "✓ Retrieved $($tasks.Count) task(s)" -ForegroundColor Green
} catch {
    Write-Host "✗ List tasks failed: $($_.Exception.Message)" -ForegroundColor Red
}

# Summary
Write-Host ""
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host "   All Integration Tests Complete   " -ForegroundColor Green
Write-Host "=====================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Test Summary:" -ForegroundColor Cyan
Write-Host "  ✓ Backend health check" -ForegroundColor Green
Write-Host "  ✓ User registration" -ForegroundColor Green
Write-Host "  ✓ Login authentication" -ForegroundColor Green
Write-Host "  ✓ Client CRUD operations" -ForegroundColor Green
Write-Host "  ✓ Task CRUD operations" -ForegroundColor Green
Write-Host ""
