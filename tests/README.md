# DeAI Integration Testing Suite

This directory contains comprehensive integration tests for the DeAI distributed AI computation platform. The tests verify that all components work together correctly in a complete end-to-end scenario.

## Test Coverage

### Core Components Tested

1. **Smart Contract (Rust/Near)**
   - Node registration and management
   - Task submission and completion
   - Token transfers and rewards
   - Staking and unstaking mechanisms

2. **API Gateway (Rust/Axum)**
   - Authentication and authorization
   - Rate limiting and security
   - Task management endpoints
   - Network information APIs

3. **Node Client (Rust + Python AI Engine)**
   - Task processing and execution
   - AI model integration
   - Result generation and submission
   - Health monitoring and reporting

4. **Web Dashboard (React/Next.js)**
   - User interface functionality
   - Near wallet integration
   - Real-time task updates
   - Analytics and monitoring

5. **SDKs (JavaScript/TypeScript and Python)**
   - Client library functionality
   - Authentication flows
   - Task management APIs
   - WebSocket real-time updates

## Test Files

### `integration_test.py`
Complete Python-based integration test suite that tests:
- API gateway health and functionality
- User registration and authentication
- Task submission, retrieval, and cancellation
- Network statistics and node information
- WebSocket real-time connections
- Rate limiting mechanisms
- Python SDK functionality

### `js_sdk_test.js`
JavaScript/Node.js integration tests for:
- JavaScript SDK initialization and configuration
- Authentication flows (login, API keys)
- Task management operations
- Network information retrieval
- WebSocket client functionality
- Error handling and retry logic

## Running the Tests

### Prerequisites

1. **Environment Setup**
   ```bash
   # Install Python dependencies
   pip install httpx websockets pydantic

   # Install Node.js dependencies
   npm install @deai/sdk

   # Set environment variables
   export DEAI_API_URL="http://localhost:8080"
   export DEAI_WS_URL="ws://localhost:8081"
   export DEAI_CONTRACT_ID="deai-compute.testnet"
   ```

2. **Start Required Services**
   ```bash
   # Start API Gateway
   cd api-gateway
   cargo run

   # Start WebSocket server (if separate)
   cd websocket-server
   cargo run

   # Start at least one node client
   cd node-client
   cargo run -- register --stake 1.0
   cargo run -- start
   ```

### Running Python Integration Tests

```bash
# Run complete integration test suite
python tests/integration_test.py

# Run with verbose logging
RUST_LOG=debug python tests/integration_test.py

# Run against specific environment
DEAI_API_URL="https://api.deai.org" python tests/integration_test.py
```

### Running JavaScript SDK Tests

```bash
# Run JavaScript SDK tests
node tests/js_sdk_test.js

# Run with specific configuration
DEAI_API_URL="http://localhost:8080" node tests/js_sdk_test.js
```

### Running All Tests

```bash
# Run both test suites
./tests/run_all_tests.sh
```

## Test Scenarios

### 1. Basic Flow Test
- User registration and authentication
- Task submission with simple text generation
- Task status monitoring until completion
- Result retrieval and validation

### 2. Advanced Task Processing
- Multiple task types (inference, classification, embedding)
- Different model configurations
- Priority and cost management
- Concurrent task processing

### 3. Network Operations
- Node registration and activation
- Node health monitoring
- Network statistics retrieval
- Node load balancing

### 4. Real-time Features
- WebSocket connection establishment
- Real-time task status updates
- Live network statistics
- Node status change notifications

### 5. Error Handling
- Invalid task parameters
- Authentication failures
- Network timeouts
- Node failures and recovery

### 6. Performance Testing
- Rate limiting validation
- Concurrent user simulation
- High-throughput task processing
- Resource usage monitoring

## Test Data and Fixtures

### Sample Task Requests

```python
# Text Generation Task
{
    "task_type": "text_generation",
    "model_name": "gpt2",
    "input_data": "The future of artificial intelligence is",
    "max_cost": "0.1",
    "priority": 5,
    "parameters": {
        "max_length": 100,
        "temperature": 0.7
    }
}

# Classification Task
{
    "task_type": "classification",
    "model_name": "cardiffnlp/twitter-roberta-base-sentiment-latest",
    "input_data": "I love this new AI technology!",
    "max_cost": "0.05"
}

# Embedding Task
{
    "task_type": "embedding",
    "model_name": "sentence-transformers/all-MiniLM-L6-v2",
    "input_data": "DeAI is a decentralized AI computation network.",
    "max_cost": "0.02"
}
```

### Test User Data

```python
{
    "username": "test_user_12345",
    "email": "test_12345@deai.test",
    "password": "test_password_123",
    "near_account_id": "test_12345.testnet"
}
```

## Expected Results

### Successful Test Run Output

```
🚀 Starting DeAI Integration Test Suite
============================================================
🔍 Testing API health...
✅ API health check passed
🔍 Testing user registration...
✅ User registration passed
🔍 Testing user login...
✅ User login passed
🔍 Testing API key creation...
✅ API key creation passed
🔍 Testing task submission...
✅ Task submission passed (ID: a1b2c3d4...)
🔍 Testing task retrieval...
✅ Task retrieval passed (Status: pending)
🔍 Testing task cancellation...
✅ Task cancellation passed
🔍 Testing network statistics...
✅ Network stats passed (Active nodes: 3)
🔍 Testing node listing...
✅ Node listing passed (3 nodes found)
🔍 Testing WebSocket connection...
✅ WebSocket connection passed
🔍 Testing rate limiting...
✅ Rate limiting test passed
🔍 Testing Python SDK...
✅ Python SDK test passed
============================================================
🎯 Test Results: 11/11 tests passed
✅ All integration tests passed!
```

## Troubleshooting

### Common Issues

1. **Connection Refused**
   - Ensure API gateway is running on the specified port
   - Check firewall settings and port availability

2. **Authentication Errors**
   - Verify test user credentials
   - Check JWT secret configuration

3. **Task Processing Timeouts**
   - Ensure at least one node client is running and registered
   - Check node client AI engine setup

4. **WebSocket Connection Failures**
   - Verify WebSocket server is running
   - Check for proxy or firewall blocking WebSocket connections

### Debug Mode

```bash
# Enable debug logging for detailed error information
export RUST_LOG=debug
export PYTHONPATH=$PWD/sdks/python
python tests/integration_test.py
```

### Manual Testing

For manual testing and debugging, you can also use:

```bash
# Test API health
curl http://localhost:8080/health

# Test task submission (with auth token)
curl -X POST http://localhost:8080/api/v1/tasks \
  -H "Authorization: Bearer YOUR_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"task_type":"text_generation","model_name":"gpt2","input_data":"Hello world"}'
```

## Continuous Integration

These tests can be integrated into CI/CD pipelines:

```yaml
# GitHub Actions example
name: Integration Tests
on: [push, pull_request]
jobs:
  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Setup Near CLI
        run: npm install -g near-cli
      - name: Deploy Smart Contract
        run: ./scripts/deploy_contract.sh
      - name: Start Services
        run: ./scripts/start_services.sh
      - name: Run Integration Tests
        run: |
          python tests/integration_test.py
          node tests/js_sdk_test.js
```

## Performance Benchmarks

The integration tests also serve as performance benchmarks:

- **API Response Time**: < 100ms for simple endpoints
- **Task Submission**: < 500ms end-to-end
- **WebSocket Latency**: < 50ms for real-time updates
- **Concurrent Users**: Support for 100+ simultaneous connections
- **Throughput**: 1000+ tasks per hour with 3 active nodes

## Contributing

When adding new features, please:

1. Add corresponding integration tests
2. Update the test documentation
3. Ensure all existing tests continue to pass
4. Add performance benchmarks if applicable

## Security Testing

The integration tests include security validations:

- Authentication bypass attempts
- Rate limiting enforcement
- Input validation and sanitization
- Access control verification
- Token expiration handling