//! Application state management

use crate::{
    analytics::AnalyticsEngine,
    anomaly_detector::AnomalyDetector,
    config::Settings,
    decision_engine::DecisionEngine,
    resource_manager::ResourceManager,
    self_healing::SelfHealingService,
};
use dashmap::DashMap;
use error_stack::{Report, ResultExt};
use parking_lot::RwLock;
use redis_interface::RedisConnectionPool;
use std::sync::Arc;
use uuid::Uuid;

/// Application state error
#[derive(Debug, thiserror::Error)]
pub enum StateError {
    /// Initialization error
    #[error("Failed to initialize state: {0}")]
    Initialization(String),

    /// Redis connection error
    #[error("Redis connection error: {0}")]
    RedisConnection(String),
}

/// Main application state
#[derive(Clone)]
pub struct AppState {
    /// Configuration
    pub config: Settings,

    /// Redis connection pool
    pub redis_pool: Arc<RedisConnectionPool>,

    /// Decision engine
    pub decision_engine: Arc<RwLock<DecisionEngine>>,

    /// Anomaly detector
    pub anomaly_detector: Arc<RwLock<AnomalyDetector>>,

    /// Self-healing service
    pub self_healing: Arc<RwLock<SelfHealingService>>,

    /// Analytics engine
    pub analytics: Arc<RwLock<AnalyticsEngine>>,

    /// Resource manager
    pub resource_manager: Arc<RwLock<ResourceManager>>,

    /// Active sessions
    pub sessions: Arc<DashMap<Uuid, SessionData>>,

    /// System metrics cache
    pub metrics_cache: Arc<RwLock<MetricsCache>>,
}

/// Session data
#[derive(Debug, Clone)]
pub struct SessionData {
    /// Session ID
    pub id: Uuid,

    /// Created at
    pub created_at: time::OffsetDateTime,

    /// Last activity
    pub last_activity: time::OffsetDateTime,

    /// Session metadata
    pub metadata: std::collections::HashMap<String, String>,
}

/// Metrics cache
#[derive(Debug, Clone, Default)]
pub struct MetricsCache {
    /// Recent payment success rate
    pub payment_success_rate: f64,

    /// Average latency
    pub avg_latency_ms: f64,

    /// Active payments count
    pub active_payments: u64,

    /// System health score (0-100)
    pub health_score: f64,

    /// Last updated
    pub last_updated: Option<time::OffsetDateTime>,
}

impl AppState {
    /// Create new application state
    pub async fn new(config: Settings) -> Result<Self, Report<StateError>> {
        // Initialize Redis connection
        let redis_pool = Self::create_redis_pool(&config)
            .await
            .change_context(StateError::RedisConnection("Failed to create Redis pool".to_string()))?;

        // Initialize components
        let decision_engine = Arc::new(RwLock::new(DecisionEngine::new(config.clone())));
        let anomaly_detector = Arc::new(RwLock::new(AnomalyDetector::new(config.clone())));
        let self_healing = Arc::new(RwLock::new(SelfHealingService::new(config.clone())));
        let analytics = Arc::new(RwLock::new(AnalyticsEngine::new(config.clone())));
        let resource_manager = Arc::new(RwLock::new(ResourceManager::new(config.clone())));

        Ok(Self {
            config,
            redis_pool: Arc::new(redis_pool),
            decision_engine,
            anomaly_detector,
            self_healing,
            analytics,
            resource_manager,
            sessions: Arc::new(DashMap::new()),
            metrics_cache: Arc::new(RwLock::new(MetricsCache::default())),
        })
    }

    /// Create Redis connection pool
    async fn create_redis_pool(config: &Settings) -> Result<RedisConnectionPool, Report<StateError>> {
        let redis_url = &config.redis.url;

        // Parse Redis configuration
        let redis_config = redis_interface::RedisSettings {
            host: redis_url.clone(),
            pool_size: config.redis.pool_size,
            reconnect_max_attempts: 5,
            reconnect_delay: 5,
            default_ttl: config.redis.default_ttl,
            stream_read_count: 1,
            default_hash_ttl: config.redis.default_ttl,
            default_command_timeout: Some(30),
            use_legacy_version: Some(false),
            disable_auto_backpressure: false,
        };

        RedisConnectionPool::new(redis_config)
            .await
            .change_context(StateError::RedisConnection("Failed to connect to Redis".to_string()))
    }

    /// Update metrics cache
    pub fn update_metrics(&self, metrics: MetricsCache) {
        let mut cache = self.metrics_cache.write();
        *cache = metrics;
    }

    /// Get current health score
    pub fn get_health_score(&self) -> f64 {
        self.metrics_cache.read().health_score
    }

    /// Create new session
    pub fn create_session(&self) -> Uuid {
        let session_id = Uuid::new_v4();
        let now = time::OffsetDateTime::now_utc();

        self.sessions.insert(session_id, SessionData {
            id: session_id,
            created_at: now,
            last_activity: now,
            metadata: std::collections::HashMap::new(),
        });

        session_id
    }

    /// Get session
    pub fn get_session(&self, session_id: &Uuid) -> Option<SessionData> {
        self.sessions.get(session_id).map(|s| s.clone())
    }

    /// Update session activity
    pub fn update_session_activity(&self, session_id: &Uuid) {
        if let Some(mut session) = self.sessions.get_mut(session_id) {
            session.last_activity = time::OffsetDateTime::now_utc();
        }
    }

    /// Clean up expired sessions
    pub fn cleanup_sessions(&self, max_age_seconds: i64) {
        let now = time::OffsetDateTime::now_utc();
        self.sessions.retain(|_, session| {
            let age = (now - session.last_activity).whole_seconds();
            age < max_age_seconds
        });
    }
}
