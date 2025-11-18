# Autonomous Orchestrator - Quick Start Guide

Get APOS up and running in 5 minutes!

## Prerequisites

- Docker and Docker Compose
- Or: Rust 1.85+, PostgreSQL 14+, Redis 7+

## Option 1: Docker (Recommended)

The fastest way to get started:

```bash
# 1. Navigate to docker directory
cd docker

# 2. Start all services
docker-compose -f docker-compose.autonomous.yml up -d

# 3. Wait for services to be healthy (30-60 seconds)
docker-compose -f docker-compose.autonomous.yml ps

# 4. Check health
curl http://localhost:8090/api/v1/health

# 5. View Grafana dashboard
open http://localhost:3000
# Login: admin / admin
```

## Option 2: Local Development

For local development:

```bash
# 1. Set environment variables
export DATABASE_URL="postgresql://user:pass@localhost/hyperswitch_db"
export REDIS_URL="redis://localhost:6379"

# 2. Run migrations
diesel migration run

# 3. Build and run
cd crates/autonomous_orchestrator
cargo run --release
```

## Verify Installation

Test the endpoints:

```bash
# Health check
curl http://localhost:8090/api/v1/health | jq

# System status
curl http://localhost:8090/api/v1/status | jq

# Analytics summary
curl http://localhost:8090/api/v1/analytics/summary | jq

# Recent anomalies
curl http://localhost:8090/api/v1/anomalies?limit=10 | jq

# Healing actions
curl http://localhost:8090/api/v1/healing/actions?limit=10 | jq
```

## Run Example Client

```bash
cd crates/autonomous_orchestrator
cargo run --example basic_usage
```

## Configuration

Edit `config/autonomous_orchestrator.toml`:

```toml
[server]
port = 8090

[decision_engine]
enable_ml_routing = true
confidence_threshold = 0.75

[self_healing]
enabled = true
auto_switch_connectors = true

[resource_manager]
enable_auto_scaling = true
cpu_scale_up_threshold = 75.0
```

## Monitoring

Access monitoring dashboards:

- **Grafana**: http://localhost:3000
- **Prometheus**: http://localhost:9090

## Integration with Hyperswitch

To integrate with existing Hyperswitch:

1. Configure Redis stream connection
2. Enable event publishing in Hyperswitch
3. Point APOS to same PostgreSQL database
4. Start APOS service

## Troubleshooting

### Service won't start
```bash
# Check logs
docker-compose -f docker/docker-compose.autonomous.yml logs autonomous_orchestrator

# Check dependencies
docker-compose -f docker/docker-compose.autonomous.yml ps
```

### Cannot connect to database
```bash
# Verify database is running
docker-compose -f docker/docker-compose.autonomous.yml exec postgres pg_isready

# Check connection string
echo $DATABASE_URL
```

### Cannot connect to Redis
```bash
# Verify Redis is running
docker-compose -f docker/docker-compose.autonomous.yml exec redis redis-cli ping

# Check connection string
echo $REDIS_URL
```

## Next Steps

- Read the [full documentation](README.md)
- Configure [monitoring alerts](docs/monitoring.md)
- Set up [production deployment](docs/production.md)
- Explore [API endpoints](docs/api.md)

## Support

- GitHub Issues: https://github.com/juspay/hyperswitch/issues
- Documentation: https://docs.hyperswitch.io
- Community: https://hyperswitch.io/community

---

Happy orchestrating! 🚀
