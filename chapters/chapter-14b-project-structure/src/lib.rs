// Capítulo 14B. Project Structure — Module organization patterns
// This crate documents the recommended structure for a Bevy game project.
// Tests verify the logical organization rather than file structure.

/// Module visibility rules
pub mod rules {
    /// A module should have a single clear responsibility
    pub fn is_single_responsibility(module_name: &str) -> bool {
        let multi_concern_patterns = ["and", "or", "misc", "util", "stuff", "common"];
        !multi_concern_patterns.iter().any(|p| module_name.to_lowercase().contains(p))
    }

    /// Check if a module depth is within recommended limits
    pub fn is_acceptable_depth(depth: usize) -> bool {
        depth <= 4 // Max 4 levels of nesting
    }
}

/// Plugin dependency graph
#[derive(Debug, Default)]
pub struct DependencyGraph {
    edges: Vec<(String, String)>, // (depends_on, module)
}

impl DependencyGraph {
    pub fn new() -> Self {
        Self { edges: Vec::new() }
    }

    pub fn add_dependency(&mut self, module: &str, depends_on: &str) {
        self.edges.push((depends_on.to_string(), module.to_string()));
    }

    /// Detect circular dependencies using DFS
    pub fn has_cycle(&self) -> bool {
        let modules: std::collections::HashSet<&str> = self.edges
            .iter()
            .flat_map(|(a, b)| vec![a.as_str(), b.as_str()])
            .collect();

        for &start in &modules {
            let mut visited = std::collections::HashSet::new();
            let mut stack = vec![start];

            while let Some(node) = stack.pop() {
                if node == start && !visited.is_empty() {
                    return true;
                }
                if visited.contains(node) {
                    continue;
                }
                visited.insert(node);

                for (from, to) in &self.edges {
                    if from == node {
                        stack.push(to);
                    }
                }
            }
        }
        false
    }

    /// Get topological order of modules
    pub fn build_order(&self) -> Result<Vec<String>, String> {
        if self.has_cycle() {
            return Err("Cannot build order: cycle detected".to_string());
        }

        let mut in_degree: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
        let mut adj: std::collections::HashMap<String, Vec<String>> = std::collections::HashMap::new();

        for (from, to) in &self.edges {
            *in_degree.entry(to.clone()).or_insert(0) += 1;
            adj.entry(from.clone()).or_default().push(to.clone());
            in_degree.entry(from.clone()).or_insert(0);
        }

        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, deg)| **deg == 0)
            .map(|(k, _)| k.clone())
            .collect();

        queue.sort();
        let mut result = Vec::new();

        while let Some(node) = queue.pop() {
            result.push(node.clone());
            if let Some(neighbors) = adj.get(&node) {
                for n in neighbors {
                    if let Some(deg) = in_degree.get_mut(n) {
                        *deg -= 1;
                        if *deg == 0 {
                            queue.push(n.clone());
                        }
                    }
                }
            }
        }

        if result.len() != in_degree.len() {
            return Err("Cycle detected during build order".to_string());
        }

        Ok(result)
    }
}

/// Recommended file size limits (in lines)
pub struct FileSizeRules;

impl FileSizeRules {
    pub const RECOMMENDED_MAX: usize = 300;
    pub const HARD_MAX: usize = 500;

    pub fn check(lines: usize) -> FileSizeStatus {
        if lines <= Self::RECOMMENDED_MAX {
            FileSizeStatus::Good
        } else if lines <= Self::HARD_MAX {
            FileSizeStatus::Warning
        } else {
            FileSizeStatus::TooLarge
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum FileSizeStatus {
    Good,
    Warning,
    TooLarge,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn single_responsibility_good_names() {
        assert!(rules::is_single_responsibility("combat"));
        assert!(rules::is_single_responsibility("rendering"));
        assert!(rules::is_single_responsibility("audio_system"));
    }

    #[test]
    fn single_responsibility_bad_names() {
        assert!(!rules::is_single_responsibility("misc"));
        assert!(!rules::is_single_responsibility("utils_and_helpers"));
        assert!(!rules::is_single_responsibility("common_stuff"));
    }

    #[test]
    fn acceptable_depth() {
        assert!(rules::is_acceptable_depth(1));
        assert!(rules::is_acceptable_depth(3));
        assert!(!rules::is_acceptable_depth(5));
        assert!(!rules::is_acceptable_depth(10));
    }

    #[test]
    fn dependency_graph_no_cycle() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("rendering", "core");
        graph.add_dependency("gameplay", "core");
        graph.add_dependency("ui", "rendering");

        assert!(!graph.has_cycle());
    }

    #[test]
    fn dependency_graph_with_cycle() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("a", "b");
        graph.add_dependency("b", "c");
        graph.add_dependency("c", "a");

        assert!(graph.has_cycle());
    }

    #[test]
    fn build_order_linear() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("rendering", "core");
        graph.add_dependency("gameplay", "rendering");

        let order = graph.build_order().unwrap();
        let core_idx = order.iter().position(|m| m == "core").unwrap();
        let render_idx = order.iter().position(|m| m == "rendering").unwrap();
        let gameplay_idx = order.iter().position(|m| m == "gameplay").unwrap();

        assert!(core_idx < render_idx, "core should come before rendering");
        assert!(render_idx < gameplay_idx, "rendering should come before gameplay");
    }

    #[test]
    fn build_order_cycle_fails() {
        let mut graph = DependencyGraph::new();
        graph.add_dependency("a", "b");
        graph.add_dependency("b", "a");

        assert!(graph.build_order().is_err());
    }

    #[test]
    fn file_size_good() {
        assert_eq!(FileSizeRules::check(100), FileSizeStatus::Good);
        assert_eq!(FileSizeRules::check(300), FileSizeStatus::Good);
    }

    #[test]
    fn file_size_warning() {
        assert_eq!(FileSizeRules::check(400), FileSizeStatus::Warning);
    }

    #[test]
    fn file_size_too_large() {
        assert_eq!(FileSizeRules::check(600), FileSizeStatus::TooLarge);
        assert_eq!(FileSizeRules::check(2000), FileSizeStatus::TooLarge);
    }
}
