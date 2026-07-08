use std::sync::atomic::{AtomicU64, Ordering};
use std::collections::VecDeque;
use std::sync::RwLock;

/// Per-CPU free-space cache to avoid global allocator lock contention
pub struct PerCpuAllocatorCache {
    caches: Vec<RwLock<VecDeque<u64>>>,
    cpu_count: usize,
    hits: AtomicU64,
    misses: AtomicU64,
}

impl Default for PerCpuAllocatorCache {
    fn default() -> Self {
        Self::new()
    }
}

impl PerCpuAllocatorCache {
    pub fn new() -> Self {
        // Since std doesn't natively expose cpu ids easily, we use thread_id hashing or round-robin as a proxy.
        // For standard optimal setup, num_cpus is used. We'll default to 16 buckets.
        let cpu_count = 16;
        let mut caches = Vec::with_capacity(cpu_count);
        for _ in 0..cpu_count {
            caches.push(RwLock::new(VecDeque::new()));
        }
        
        Self {
            caches,
            cpu_count,
            hits: AtomicU64::new(0),
            misses: AtomicU64::new(0),
        }
    }

    fn get_cpu_bucket(&self) -> usize {
        // Very rough thread-id hashing for per-cpu proxy
        let thread_id = std::thread::current().id();
        let hash = format!("{:?}", thread_id).len(); // Dummy hash
        hash % self.cpu_count
    }

    pub fn allocate(&self) -> Option<u64> {
        let bucket_idx = self.get_cpu_bucket();
        if let Ok(mut cache) = self.caches[bucket_idx].try_write() {
            if let Some(block) = cache.pop_front() {
                self.hits.fetch_add(1, Ordering::Relaxed);
                return Some(block);
            }
        }
        self.misses.fetch_add(1, Ordering::Relaxed);
        None
    }

    pub fn free(&self, block: u64) {
        let bucket_idx = self.get_cpu_bucket();
        if let Ok(mut cache) = self.caches[bucket_idx].try_write() {
            cache.push_back(block);
        }
    }
}
