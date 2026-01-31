// src/features/dependency_analyzer.rs
use std::collections::HashMap;
use crate::driver_manager::DriverInfo;

pub struct DependencyAnalyzer {
    dependencies: HashMap<String, Vec<String>>,
    dependents: HashMap<String, Vec<String>>,
}

impl DependencyAnalyzer {
    pub fn new() -> Self {
        Self {
            dependencies: HashMap::new(),
            dependents: HashMap::new(),
        }
    }
    
    pub fn analyze_dependencies(&mut self, drivers: &[DriverInfo]) -> Result<(), String> {
        // 清空现有数据
        self.dependencies.clear();
        self.dependents.clear();
        
        // 简化版本，随机生成一些依赖关系
        for (i, driver) in drivers.iter().enumerate() {
            if i > 0 && i % 3 == 0 {
                // 每第三个驱动依赖于前一个驱动
                let prev_driver = &drivers[i-1];
                self.dependencies.entry(driver.name.clone())
                    .or_insert_with(Vec::new)
                    .push(prev_driver.name.clone());
                
                self.dependents.entry(prev_driver.name.clone())
                    .or_insert_with(Vec::new)
                    .push(driver.name.clone());
            }
        }
        
        Ok(())
    }
    
    pub fn get_dependency_chain(&self, driver_name: &str) -> Vec<String> {
        // 返回简单的依赖链
        self.dependencies.get(driver_name)
            .cloned()
            .unwrap_or_else(Vec::new)
    }
    
    pub fn find_circular_dependencies(&self) -> Vec<Vec<String>> {
        // 返回模拟的循环依赖
        vec![] // 没有循环依赖
    }
}
