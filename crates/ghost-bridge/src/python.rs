//! Python bridge via PyO3
//!
//! Provides FFI integration with Python polyglot workers.
//! Workers are loaded based on their manifest.json configuration.

use ghost_schema::{GhostError, PayloadBlob, RawContext, PolyglotManifest};
use std::path::{Path, PathBuf};
use std::collections::HashMap;

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
    pub fn from_manifest(manifest: PolyglotManifest, worker_path: PathBuf) -> Result<Self, GhostError> {
        let module_name = manifest.runtime.module.clone()
            .or_else(|| {
                // Derive module name from entrypoint
                let parts: Vec<&str> = manifest.runtime.entrypoint.split(':').collect();
                parts.first().map(|s| s.to_string())
            })
            .ok_or_else(|| GhostError::ConfigError(
                "Could not determine module name from manifest".into()
            ))?;
        
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
    pub fn capabilities(&self) -> &[String] {
        &self.manifest.capabilities.iter()
            .map(|c| c.name.clone())
            .collect::<Vec<_>>()
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
            Err(GhostError::NotImplemented("Python bridge not enabled (pyo3 feature required)".into()))
        }
    }
    
    #[cfg(feature = "pyo3")]
    fn execute_pyo3(&self, ctx: &RawContext) -> Result<PayloadBlob, GhostError> {
        use pyo3::prelude::*;
        
        Python::with_gil(|py| {
            // Add worker path to sys.path if not already present
            let sys = py.import("sys")?;
            let path: &PyList = sys.getattr("path")?.downcast()?;
            let path_str = self.worker_path.join("src").to_string_lossy().to_string();
            
            // Check if path already exists
            let contains = path.iter().any(|p| {
                p.extract::<String>().map(|s| s == path_str).unwrap_or(false)
            });
            
            if !contains {
                path.insert(0, &path_str)?;
            }
            
            // Import the worker module
            let module = py.import(&self.module_name)
                .map_err(|e| GhostError::ScraperError {
                    worker: self.manifest.id.clone(),
                    message: format!("Failed to import module '{}': {}", self.module_name, e),
                })?;
            
            // Get the execute function from FFI config
            let execute_fn_name = self.manifest.get_ffi_entry("execute")
                .unwrap_or("execute_worker");
            
            let worker_fn = module.getattr(execute_fn_name)
                .map_err(|e| GhostError::ScraperError {
                    worker: self.manifest.id.clone(),
                    message: format!("Function '{}' not found: {}", execute_fn_name, e),
                })?;
            
            // Serialize context to JSON
            let context_json = serde_json::to_string(ctx)
                .map_err(|e| GhostError::SerializationError(e.to_string()))?;
            
            // Call the worker function
            let result = worker_fn.call1((context_json,))
                .map_err(|e| GhostError::ScraperError {
                    worker: self.manifest.id.clone(),
                    message: format!("Execution failed: {}", e),
                })?;
            
            // Extract result as JSON string
            let result_json: String = result.extract()
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
                
                if !path.iter().any(|p| p.extract::<String>().map(|s| s == path_str).unwrap_or(false)) {
                    path.insert(0, &path_str)?;
                }
                
                let module = py.import(&self.module_name)?;
                let info_fn_name = self.manifest.get_ffi_entry("info").unwrap_or("get_worker_info");
                let info_fn = module.getattr(info_fn_name)?;
                let result = info_fn.call0()?;
                let info_json: String = result.extract()?;
                
                serde_json::from_str(&info_json)
                    .map_err(|e| GhostError::ParseError(e.to_string()))
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
                
                if !path.iter().any(|p| p.extract::<String>().map(|s| s == path_str).unwrap_or(false)) {
                    path.insert(0, &path_str)?;
                }
                
                let module = py.import(&self.module_name)?;
                let health_fn_name = self.manifest.get_ffi_entry("health").unwrap_or("health_check");
                let health_fn = module.getattr(health_fn_name)?;
                let result = health_fn.call1((deep,))?;
                let health_json: String = result.extract()?;
                
                serde_json::from_str(&health_json)
                    .map_err(|e| GhostError::ParseError(e.to_string()))
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
    ctx: &RawContext,
) -> Result<PayloadBlob, GhostError> {
    // TODO: Look up worker by ID from registry
    Err(GhostError::NotImplemented(format!(
        "Worker '{}' not found in registry",
        worker_id
    )))
}

/// Python bridge implementation
pub struct PythonBridge {
    /// Whether initialized
    initialized: bool,
    /// Loaded workers by ID
    workers: HashMap<String, PythonWorker>,
    /// Worker modules (legacy)
    modules: Vec<String>,
}

impl PythonBridge {
    /// Creates a new Python bridge
    pub fn new() -> Result<Self, GhostError> {
        Ok(Self {
            initialized: false,
            workers: HashMap::new(),
            modules: Vec::new(),
        })
    }

    /// Initializes the Python runtime
    pub fn initialize(&mut self) -> Result<(), GhostError> {
        #[cfg(feature = "pyo3")]
        {
            pyo3::prepare_freethreaded_python();
        }

        self.initialized = true;
        Ok(())
    }

    /// Load a polyglot worker from directory
    pub fn load_worker(&mut self, worker_dir: &Path) -> Result<&PythonWorker, GhostError> {
        let worker = PythonWorker::load_from_dir(worker_dir)?;
        let id = worker.id().to_string();
        self.workers.insert(id.clone(), worker);
        Ok(self.workers.get(&id).unwrap())
    }
    
    /// Load all workers from scrapers directory
    pub fn load_workers_from_scrapers_dir(&mut self, base_dir: &Path) -> Result<Vec<String>, GhostError> {
        let mut loaded = Vec::new();
        
        let scrapers_dir = base_dir.join("scrapers");
        if !scrapers_dir.exists() {
            return Ok(loaded);
        }
        
        for entry in std::fs::read_dir(&scrapers_dir)
            .map_err(|e| GhostError::ConfigError(format!("Failed to read scrapers dir: {}", e)))? 
        {
            let entry = entry.map_err(|e| GhostError::ConfigError(e.to_string()))?;
            let path = entry.path();
            
            // Check if directory contains manifest.json
            if path.is_dir() && path.join("manifest.json").exists() {
                match self.load_worker(&path) {
                    Ok(worker) => {
                        loaded.push(worker.id().to_string());
                    }
                    Err(e) => {
                        eprintln!("Failed to load worker from {:?}: {}", path, e);
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
        let worker = self.workers.get(id)
            .ok_or_else(|| GhostError::ScraperError {
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
    pub fn shutdown(&mut self) -> Result<(), GhostError> {
        self.initialized = false;
        self.workers.clear();
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
}

impl Default for PythonBridge {
    fn default() -> Self {
        Self::new().unwrap_or(Self {
            initialized: false,
            workers: HashMap::new(),
            modules: Vec::new(),
        })
    }
}
