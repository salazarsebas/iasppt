# DeAI Node Client

A Rust-based node client for the DeAI (Decentralized AI) computation network built on Near Protocol.

## Overview

The DeAI Node Client enables your hardware to participate in the decentralized AI computation network by:

- Registering as a compute node with collateral staking
- Executing AI inference and training tasks
- Earning rewards in DeAI tokens
- Contributing to the distributed AI infrastructure

## Architecture

- **Rust Core**: Near blockchain integration, task management, networking
- **Python AI Engine**: Model execution using PyTorch/Transformers
- **Hybrid Design**: Efficient resource utilization and flexibility

## Requirements

### Software
- Rust 1.70+ with cargo
- Python 3.8+ with pip
- Git

### Hardware (Recommended)
- NVIDIA GPU with 8GB+ VRAM
- 16GB+ RAM
- 100GB+ free storage
- Stable internet connection

### Near Protocol
- Testnet account with NEAR tokens for staking
- Private key for transaction signing

## Quick Start

### 1. Setup

```bash
git clone <repository>
cd node-client
./setup.sh
```

### 2. Configuration

Edit `node_config.toml`:

```toml
[node]
account_id = "your-node.testnet"
private_key = "ed25519:YOUR_PRIVATE_KEY"
public_ip = "YOUR_PUBLIC_IP"
stake_amount = "5.0"  # NEAR tokens to stake

[hardware]
gpu_specs = "NVIDIA RTX 4090"
cpu_specs = "Intel i9-13900K"
max_concurrent_tasks = 2
```

### 3. Registration

```bash
# Register your node with the network
./target/release/deai-node-client register

# Check registration status
./target/release/deai-node-client status
```

### 4. Start Node

```bash
# Start the node daemon
./target/release/deai-node-client start
```

## Commands

### Register Node
Register your hardware as a compute node:
```bash
deai-node-client register --config node_config.toml
```

### Start Daemon
Begin accepting and processing AI tasks:
```bash
deai-node-client start --config node_config.toml
```

### Check Status
View node information and statistics:
```bash
deai-node-client status --config node_config.toml
```

### Deactivate
Stop the node and withdraw stake:
```bash
deai-node-client deactivate --config node_config.toml
```

## Supported AI Tasks

### Text Generation
- Models: GPT-2, GPT-J, LLaMA, etc.
- Use cases: Content generation, chatbots

### Classification
- Models: BERT, RoBERTa, DistilBERT
- Use cases: Sentiment analysis, text classification

### Inference
- Models: Any Hugging Face transformer
- Use cases: General model inference

### Embeddings
- Models: Sentence transformers, BERT variants
- Use cases: Semantic search, clustering

## Configuration

### Node Settings
- `account_id`: Your Near testnet account
- `private_key`: Ed25519 private key for signing
- `public_ip`: Public IP address for API access
- `stake_amount`: NEAR tokens to stake (minimum 1.0)

### AI Settings
- `python_path`: Python interpreter path
- `models_cache_dir`: Local model storage directory
- `huggingface_token`: Optional HF API token for private models
- `max_model_size_gb`: Maximum model download size

### Hardware Settings
- `gpu_specs`: GPU model description
- `cpu_specs`: CPU model description
- `max_concurrent_tasks`: Parallel task limit

## Monitoring

### Logs
- Application logs: `RUST_LOG=info`
- AI engine logs: Check Python output

### Health Checks
- Heartbeat: Automatic every 60 seconds
- Status: `deai-node-client status`
- Near Explorer: View transactions

### Metrics
- Tasks completed
- Reputation score
- Token rewards earned
- Uptime statistics

## Troubleshooting

### Common Issues

**Registration Failed**
- Check account balance (need NEAR for gas + stake)
- Verify private key format
- Ensure unique public IP

**AI Tasks Failing**
- Check Python dependencies: `pip install -r ai_engine/requirements.txt`
- Verify GPU drivers (CUDA for NVIDIA)
- Check model download permissions

**Network Issues**
- Verify internet connectivity
- Check firewall settings for API port
- Test Near RPC connectivity

### Debug Mode
```bash
RUST_LOG=debug deai-node-client start
```

### Test AI Engine
```bash
cd ai_engine
python3 ai_worker.py '{"model": "distilbert-base-uncased", "input": "test", "task_type": "inference"}'
```

## Security

### Private Key Management
- Store private keys securely
- Use environment variables in production
- Never commit keys to version control

### Network Security
- Use firewall to restrict API access
- Consider VPN for additional security
- Monitor for unusual activity

### Model Security
- Verify model sources
- Scan for malicious code
- Use trusted model repositories

## Economics

### Rewards
- Earn tokens for completed tasks
- Higher reputation = more task assignments
- Bonus for consistent uptime

### Costs
- Initial stake (recoverable when deactivating)
- Electricity and hardware costs
- Internet bandwidth

### ROI Calculation
- Monitor task completion rates
- Track token value and rewards
- Consider hardware depreciation

## Development

### Building from Source
```bash
cargo build --release
```

### Running Tests
```bash
cargo test
python3 -m pytest ai_engine/tests/
```

### Contributing
1. Fork the repository
2. Create feature branch
3. Add tests for new functionality
4. Submit pull request

## Support

### Documentation
- [Near Protocol Docs](https://docs.near.org)
- [DeAI Documentation](https://docs.deai.org)

### Community
- Discord: [DeAI Community](https://discord.gg/deai)
- Telegram: [DeAI Developers](https://t.me/deai_dev)

### Issues
- GitHub Issues: Report bugs and feature requests
- Support Email: support@deai.org

## License

MIT License - see LICENSE file for details.

## Changelog

### v0.1.0
- Initial release
- Basic node registration and task execution
- Support for transformer models
- Near Protocol integration