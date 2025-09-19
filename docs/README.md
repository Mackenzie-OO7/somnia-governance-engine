# Introduction

Welcome to the comprehensive documentation for the Somnia Governance Engine - a modular, production-ready governance toolkit for blockchain ecosystems.

## 🎯 What is Somnia Governance Engine?

The Somnia Governance Engine is a complete governance solution that enables projects to:

* **Create and manage governance proposals** with timelock execution
* **Implement token-based voting** with proper delegation and snapshot mechanics
* **Run simple voting sessions** for community decisions
* **Integrate governance functionality** into existing Rust applications
* **Deploy secure contracts** with built-in anti-spam and security measures

## 🧩 Modular Architecture

The engine is designed as a **modular toolkit** where you can use any combination of components:

### Smart Contracts (Optional)

* **Use Ours**: Production-ready governance contracts
* **Use Yours**: Point our Rust engine to your existing contracts
* **Use None**: Pure off-chain governance with cryptographic signatures

### Storage Layer (Optional)

* **PostgreSQL**: Full-featured database with our schema
* **Your Database**: Use existing database with custom adapters
* **File Storage**: Simple file-based persistence
* **In-Memory**: No persistence, great for testing

## 🚀 Quick Start

Choose your integration approach:

### Minimal Setup (Pure Rust)

```bash
# Just add to your Cargo.toml - no other dependencies needed
cargo add somnia-governance-engine
```

### With Smart Contracts

```bash
# Requires Node.js + Foundry for contract deployment
git clone https://github.com/your-org/somnia-governance-engine.git
cd contracts && forge build
```

### With Database Persistence

```bash
# Requires PostgreSQL
createdb governance_db
# Configure DATABASE_URL in .env
```

## 📚 Documentation Structure

This documentation is organized into several comprehensive guides:

* [**Architecture Overview**](ARCHITECTURE_OVERVIEW.md) - Understanding the modular design
* [**API Guide**](integration-guides/api_guide.md) - Complete Rust integration reference
* [**Smart Contracts Guide**](SMART_CONTRACTS_GUIDE.md) - Contract deployment and interaction
* [**Integration Examples**](INTEGRATION_EXAMPLES.md) - Real-world implementation patterns
* [**Governance Best Practices**](GOVERNANCE_BEST_PRACTICES.md) - Design patterns and workflows

## 🎯 Choose Your Path

### For Ecosystem Projects

* Start with [Architecture Overview](ARCHITECTURE_OVERVIEW.md) to understand integration options
* Follow [Integration Examples](INTEGRATION_EXAMPLES.md) for your specific use case

### For Developers

* Check [API Guide](integration-guides/api_guide.md) for comprehensive API documentation
* Use [Integration Examples](INTEGRATION_EXAMPLES.md) for code templates

### For Governance Designers

* Read [Governance Best Practices](GOVERNANCE_BEST_PRACTICES.md) for proven patterns
* Review [Architecture Overview](ARCHITECTURE_OVERVIEW.md) for capability overview

## 🔗 External Resources

* **GitHub Repository**: [Somnia Governance Engine](https://github.com/your-org/somnia-governance-engine)
* **Somnia Network**: [Official Documentation](https://docs.somnia.network)
* **Community Discord**: [Join Discussion](https://discord.gg/somnia)

## 🆘 Getting Help

* **Issues**: Report bugs on [GitHub Issues](https://github.com/your-org/somnia-governance-engine/issues)
* **Discussions**: Ask questions in [GitHub Discussions](https://github.com/your-org/somnia-governance-engine/discussions)
* **Community**: Join our [Discord](https://discord.gg/somnia) for real-time help

***

**Ready to get started?** Choose your integration path above or dive into the [Architecture Overview](ARCHITECTURE_OVERVIEW.md)!
