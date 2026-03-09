//! Python bridge via PyO3
//!
//! Provides FFI integration with Python polyglot workers.
//! Workers are loaded based on their manifest.json configuration.

use ghost_schema::{BridgeStats, BridgeType, GhostError, PayloadBlob, PolyglotManifest, RawContext};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::bridge::Bridge;

/// Python polyglot worker wrapper
pub struct PythonWorker {
    /// Worker manifest
    manifest: PolyglotManifest,
    /// Module name for import
    module_name: String,
    /// Path to worker directory
    worker_path: PathBuf,
}

impl PythonWorker {
    /// Create a new Python worker from manifest
    pub fn from_manifest(
        manifest: PolyglotManifest,
        worker_path: PathBuf,
    ) -> Result<Self, GhostError> {
        let module_name = manifest
            .runtime
            .module
            .clone()
            .or_else(|| {
                // Derive module name from entrypoint
                let parts: Vec<&str> = manifest.runtime.entrypoint.split(':').collect();
                parts.first().map(|s| s.to_string())
            })
            .ok_or_else(|| {
                GhostError::ConfigError("Could not determine module name from manifest".into())
            })?;

        Ok(Self {
            manifest,
            module_name,
            worker_path,
        })
    }

    /// Load worker from directory
    pub fn load_from_dir(dir: &Path) -> Result<Self, GhostError> {
        let manifest_path = dir.join("manifest.json");
        let manifest = PolyglotManifest::from_file(&manifest_path)?;
        Self::from_manifest(manifest, dir.to_path_buf())
    }

    /// Get worker ID
    pub fn id(&self) -> &str {
        &self.manifest.id
    }

    /// Get worker capabilities
    pub fn capabilities(&self) -> Vec<String> {
        self.manifest.capabilities.iter().map(|c| c.name.clone()).collect()
    }

    /// Get worker platforms
    pub fn platforms(&self) -> &[String] {
        &self.manifest.platforms
    }

    /// Get the manifest
    pub fn manifest(&self) -> &PolyglotManifest {
        &self.manifest
    }

    /// Execute the worker with given context
    pub fn execute(&self, ctx: &RawContext) -> Result<PayloadBlob, GhostError> {
        #[cfg(feature = "pyo3")]
        {
            self.execute_pyo3(ctx)
        }

        #[cfg(not(feature = "pyo3"))]
        {
            let _ = ctx;
            Err(GhostError::ConfigError(
                "Python bridge not enabled (pyo3 feature required)".into(),
            ))
        }
    }

    #[cfg(feature = "pyo3")]
    fn execute_pyo3(&self, ctx: &RawContext) -> Result<PayloadBlob, GhostError> {
        use pyo3::prelude::*;

        Python::with_gil(|py| {
            // Add worker path to sys.path if not already present
            let sys = py.import("sys")?;
            let path: &PyList = sys.getattr("path")?.downcast()?;
            let path_str = self
                .worker_path
                .join("src")
                .to_string_lossy()
                .to_string();

            // Check if path already exists
            let contains = path.iter().any(|p| {
                p.extract::<String>()
                    .map(|s| s == path_str)
                    .unwrap_or(false)
            });

            if !contains {
                path.insert(0, &path_str)?;
            }

            // Import the worker module
            let module = py.import(&self.module_name).map_err(|e| GhostError::ScraperError {
                worker: self.manifest.id.clone(),
                message: format!("Failed to import module '{}': {}", self.module_name, e),
            })?;

            // Get the execute function from FFI config
            let execute_fn_name = self
                .manifest
                .get_ffi_entry("execute")
                .unwrap_or("execute_worker");

            let worker_fn = module.getattr(execute_fn_name).map_err(|e| GhostError::ScraperError {
                worker: self.manifest.id.clone(),
                message: format!("Function '{}' not found: {}", execute_fn_name, e),
            })?;

            // Serialize context to JSON
            let context_json =
                serde_json::to_string(ctx).map_err(|e| GhostError::JsonError(e.to_string()))?;

            // Call the worker function
            let result = worker_fn.call1((context_json,)).map_err(|e| GhostError::ScraperError {
                worker: self.manifest.id.clone(),
                message: format!("Execution failed: {}", e),
            })?;

            // Extract result as JSON string
            let result_json: String = result
                .extract()
                .map_err(|e| GhostError::ParseError(format!("Invalid result format: {}", e)))?;

            // Deserialize to PayloadBlob
            let payload: PayloadBlob = serde_json::from_str(&result_json)
                .map_err(|e| GhostError::ParseError(format!("Failed to parse PayloadBlob: {}", e)))?;

            Ok(payload)
        })
    }

    /// Get worker info
    pub fn get_info(&self) -> Result<WorkerInfo, GhostError> {
        #[cfg(feature = "pyo3")]
        {
            use pyo3::prelude::*;

            Python::with_gil(|py| {
                let sys = py.import("sys")?;
                let path: &PyList = sys.getattr("path")?.downcast()?;
                let path_str = self.worker_path.join("src").to_string_lossy().to_string();

                if !path.iter().any(|p| {
                    p.extract::<String>()
                        .map(|s| s == path_str)
                        .unwrap_or(false)
                }) {
                    path.insert(0, &path_str)?;
                }

                let module = py.import(&self.module_name)?;
                let info_fn_name = self.manifest.get_ffi_entry("info").unwrap_or("get_worker_info");
                let info_fn = module.getattr(info_fn_name)?;
                let result = info_fn.call0()?;
                let info_json: String = result.extract()?;

                serde_json::from_str(&info_json).map_err(|e| GhostError::ParseError(e.to_string()))
            })
        }

        #[cfg(not(feature = "pyo3"))]
        {
            Ok(WorkerInfo {
                id: self.manifest.id.clone(),
                version: self.manifest.version.clone(),
                capabilities: self.manifest.capabilities.iter().map(|c| c.name.clone()).collect(),
                platforms: self.manifest.platforms.clone(),
            })
        }
    }

    /// Perform health check
    pub fn health_check(&self, deep: bool) -> Result<HealthStatus, GhostError> {
        #[cfg(feature = "pyo3")]
        {
            use pyo3::prelude::*;

            Python::with_gil(|py| {
                let sys = py.import("sys")?;
                let path: &PyList = sys.getattr("path")?.downcast()?;
                let path_str = self.worker_path.join("src").to_string_lossy().to_string();

                if !path.iter().any(|p| {
                    p.extract::<String>()
                        .map(|s| s == path_str)
                        .unwrap_or(false)
                }) {
                    path.insert(0, &path_str)?;
                }

                let module = py.import(&self.module_name)?;
                let health_fn_name = self.manifest.get_ffi_entry("health").unwrap_or("health_check");
                let health_fn = module.getattr(health_fn_name)?;
                let result = health_fn.call1((deep,))?;
                let health_json: String = result.extract()?;

                serde_json::from_str(&health_json).map_err(|e| GhostError::ParseError(e.to_string()))
            })
        }

        #[cfg(not(feature = "pyo3"))]
        {
            Ok(HealthStatus::default())
        }
    }
}

/// Worker info returned by get_worker_info()
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkerInfo {
    pub id: String,
    pub version: String,
    pub capabilities: Vec<String>,
    pub platforms: Vec<String>,
}

/// Health status returned by health_check()
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, Default)]
pub struct HealthStatus {
    pub status: String,
    pub score: f64,
    pub latency_ms: i64,
    pub success_rate: f64,
    pub last_check: String,
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub details: HashMap<String, serde_json::Value>,
}

/// Executes a Python worker (legacy function)
pub async fn execute_python_worker(
    worker_id: &str,
    _ctx: &RawContext,
) -> Result<PayloadBlob, GhostError> {
    // This function requires a registry lookup
    // Use PythonBridge::execute_worker instead
    Err(GhostError::NotImplemented(format!(
        "Worker '{}' not found - use PythonBridge::execute_worker instead",
        worker_id
    )))
}

/// Python bridge implementation
///
/// Manages communication with Python workers via PyO3.
/// This bridge allows calling Python scrapers from Rust as if they were native functions.
pub struct PythonBridge {
    /// Whether the bridge has been initialized
    initialized: bool,
    /// Loaded workers by ID
    workers: HashMap<String, PythonWorker>,
    /// Legacy worker modules
    modules: Vec<String>,
    /// Bridge statistics
    stats: BridgeStats,
    /// Configuration for the bridge
    config: PythonBridgeConfig,
}

/// Configuration for PythonBridge
#[derive(Debug, Clone)]
pub struct PythonBridgeConfig {
    /// Maximum number of workers
    pub max_workers: usize,
    /// Request timeout in milliseconds
    pub timeout_ms: u64,
    /// Memory limit per worker in MB
    pub memory_limit_mb: u64,
}

impl Default for PythonBridgeConfig {
    fn default() -> Self {
        Self {
            max_workers: 10,
            timeout_ms: 60000,
            memory_limit_mb: 512,
        }
    }
}

impl PythonBridge {
    /// Creates a new Python bridge with default configuration
    pub fn new() -> Result<Self, GhostError> {
        Ok(Self {
            initialized: false,
            workers: HashMap::new(),
            modules: Vec::new(),
            stats: BridgeStats::new(),
            config: PythonBridgeConfig::default(),
        })
    }

    /// Creates a new Python bridge with custom configuration
    pub fn with_config(config: PythonBridgeConfig) -> Result<Self, GhostError> {
        Ok(Self {
            initialized: false,
            workers: HashMap::new(),
            modules: Vec::new(),
            stats: BridgeStats::new(),
            config,
        })
    }

    /// Initializes the Python runtime
    pub fn initialize_bridge(&mut self) -> Result<(), GhostError> {
        #[cfg(feature = "pyo3")]
        {
            pyo3::prepare_freethreaded_python();
            tracing::info!("Initializing Python PyO3 bridge");
        }

        self.initialized = true;
        self.stats.is_initialized = true;
        Ok(())
    }

    /// Load a polyglot worker from directory
    pub fn load_worker(&mut self, worker_dir: &Path) -> Result<&PythonWorker, GhostError> {
        if self.workers.len() >= self.config.max_workers {
            return Err(GhostError::ConfigError(
                "Maximum number of workers already loaded".into(),
            ));
        }

        let worker = PythonWorker::load_from_dir(worker_dir)?;
        let id = worker.id().to_string();
        self.workers.insert(id.clone(), worker);
        self.stats.active_workers = self.workers.len();
        Ok(self.workers.get(&id).unwrap())
    }

    /// Load all workers from scrapers directory
    pub fn load_workers_from_scrapers_dir(
        &mut self,
        base_dir: &Path,
    ) -> Result<Vec<String>, GhostError> {
        let mut loaded = Vec::new();

        let scrapers_dir = base_dir.join("scrapers");
        if !scrapers_dir.exists() {
            return Ok(loaded);
        }

        let entries = std::fs::read_dir(&scrapers_dir)
            .map_err(|e| GhostError::ConfigError(format!("Failed to read scrapers dir: {}", e)))?;

        for entry in entries {
            let entry = entry.map_err(|e| GhostError::ConfigError(e.to_string()))?;
            let path = entry.path();

            // Check if directory contains manifest.json
            if path.is_dir() && path.join("manifest.json").exists() {
                match self.load_worker(&path) {
                    Ok(worker) => {
                        loaded.push(worker.id().to_string());
                    }
                    Err(e) => {
                        tracing::warn!("Failed to load worker from {:?}: {}", path, e);
                    }
                }
            }
        }

        Ok(loaded)
    }

    /// Get a loaded worker by ID
    pub fn get_worker(&self, id: &str) -> Option<&PythonWorker> {
        self.workers.get(id)
    }

    /// Execute a worker by ID
    pub fn execute_worker(&self, id: &str, ctx: &RawContext) -> Result<PayloadBlob, GhostError> {
        let worker = self.workers.get(id).ok_or_else(|| GhostError::ScraperError {
            worker: id.to_string(),
            message: "Worker not found".into(),
        })?;

        worker.execute(ctx)
    }

    /// Get all loaded worker IDs
    pub fn worker_ids(&self) -> impl Iterator<Item = &String> {
        self.workers.keys()
    }

    /// Loads a worker module (legacy)
    pub fn load_module(&mut self, module_name: &str) -> Result<(), GhostError> {
        #[cfg(feature = "pyo3")]
        {
            pyo3::Python::with_gil(|py| {
                py.import(module_name)?;
                Ok(())
            })
            .map_err(|e: pyo3::PyErr| GhostError::ConfigError(e.to_string()))?;
        }

        self.modules.push(module_name.to_string());
        Ok(())
    }

    /// Shuts down the Python runtime
    pub fn shutdown_bridge(&mut self) -> Result<(), GhostError> {
        tracing::info!("Shutting down Python PyO3 bridge");
        self.initialized = false;
        self.stats.is_initialized = false;
        self.workers.clear();
        self.modules.clear();
        self.stats.active_workers = 0;
        Ok(())
    }

    /// Returns whether the bridge is initialized
    pub fn is_initialized(&self) -> bool {
        self.initialized
    }

    /// Returns the loaded modules (legacy)
    pub fn modules(&self) -> &[String] {
        &self.modules
    }

    /// Returns the bridge configuration
    pub fn config(&self) -> &PythonBridgeConfig {
        &self.config
    }

    /// Returns mutable access to statistics
    pub fn stats_mut(&mut self) -> &mut BridgeStats {
        &mut self.stats
    }

    /// Returns the number of loaded workers
    pub fn worker_count(&self) -> usize {
        self.workers.len()
    }
}

impl Bridge for PythonBridge {
    fn bridge_type(&self) -> BridgeType {
        BridgeType::PyO3
    }

    fn initialize(&mut self) -> Result<(), GhostError> {
        self.initialize_bridge()
    }

    fn shutdown(&mut self) -> Result<(), GhostError> {
        self.shutdown_bridge()
    }

    fn is_healthy(&self) -> bool {
        self.initialized
    }

    fn stats(&self) -> BridgeStats {
        self.stats.clone()
    }
}

impl Default for PythonBridge {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            initialized: false,
            workers: HashMap::new(),
            modules: Vec::new(),
            stats: BridgeStats::new(),
            config: PythonBridgeConfig::default(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_python_bridge_new() {
        let bridge = PythonBridge::new().unwrap();
        assert!(!bridge.is_initialized());
        assert_eq!(bridge.worker_count(), 0);
    }

    #[test]
    fn test_python_bridge_initialize() {
        let mut bridge = PythonBridge::new().unwrap();
        assert!(bridge.initialize().is_ok());
        assert!(bridge.is_initialized());
    }

    #[test]
    fn test_python_bridge_shutdown() {
        let mut bridge = PythonBridge::new().unwrap();
        bridge.initialize().unwrap();
        assert!(bridge.shutdown().is_ok());
        assert!(!bridge.is_initialized());
    }

    #[test]
    fn test_python_bridge_stats() {
        let bridge = PythonBridge::new().unwrap();
        let stats = bridge.stats();
        assert_eq!(stats.active_workers, 0);
        assert!(!stats.is_initialized);
    }

    #[test]
    fn test_worker_info_default() {
        let info = WorkerInfo {
            id: "test".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec!["read".to_string()],
            platforms: vec!["x".to_string()],
        };
        assert_eq!(info.id, "test");
    }

    #[test]
    fn test_health_status_default() {
        let status = HealthStatus::default();
        assert_eq!(status.status, "");
        assert_eq!(status.score, 0.0);
    }
}
