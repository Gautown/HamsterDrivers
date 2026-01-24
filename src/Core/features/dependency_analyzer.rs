use std::collections::{HashMap, HashSet};
use crate::Core::driver_manager::DriverInfo;

pub struct DependencyAnalyzer {
    dependencies: HashMap<String, Vec<String>>,
}

impl DependencyAnalyzer {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
        }
    }

    pub fn analyze_dependencies(&mut self, drivers: &[DriverInfo]) -> Result<(), String> {
        // 简单的依赖分析算法
        for driver in drivers {
            let deps = drivers.iter()
                .filter(|d| d.name != driver.name && 
                         driver.name.contains(&d.name[..d.name.len().min(3)])) // 简单的启发式规则
                .map(|d| d.name.clone())
                .collect();
            
            self.dependencies.insert(driver.name.clone(), deps);
        }
        
        Ok(())
    }

    pub fn find_circular_dependencies(&self) -> Vec<Vec<String>> {
        let mut cycles = Vec::new();
        
        // 简化的循环依赖检测
        for (driver, deps) in &self.dependencies {
            for dep in deps {
                if let Some(reverse_deps) = self.dependencies.get(dep) {
                    if reverse_deps.contains(driver) {
                        cycles.push(vec![driver.clone(), dep.clone()]);
                    }
                }
            }
        }
        
        cycles
    }

    pub fn get_dependency_chain(&self, driver_name: &str) -> Vec<String> {
        self.dependencies.get(driver_name)
            .cloned()
            .unwrap_or_else(|| vec![])
    }
}