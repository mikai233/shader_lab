use std::borrow::Cow;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, bail};

pub struct ShaderCompiler {
    root: PathBuf,
}

impl ShaderCompiler {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn compile(
        &self,
        device: &wgpu::Device,
        relative_path: &str,
    ) -> anyhow::Result<wgpu::ShaderModule> {
        let source = self.load(relative_path)?;
        naga::front::wgsl::parse_str(&source)
            .map_err(|err| anyhow::anyhow!(err.emit_to_string(&source)))
            .with_context(|| format!("shader validation failed for {relative_path}"))?;
        Ok(device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(relative_path),
            source: wgpu::ShaderSource::Wgsl(Cow::Owned(source)),
        }))
    }

    fn load(&self, relative_path: &str) -> anyhow::Result<String> {
        let mut visited = HashSet::new();
        self.load_recursive(Path::new(relative_path), &mut visited)
    }

    fn load_recursive(
        &self,
        relative_path: &Path,
        visited: &mut HashSet<PathBuf>,
    ) -> anyhow::Result<String> {
        let resolved = self.root.join(relative_path);
        let canonical_key = relative_path.to_path_buf();
        if !visited.insert(canonical_key.clone()) {
            bail!(
                "cyclic shader include detected at {}",
                canonical_key.display()
            );
        }

        let text = fs::read_to_string(&resolved)
            .with_context(|| format!("failed to read shader source {}", resolved.display()))?;

        let mut output = String::new();
        let include_base = relative_path.parent().unwrap_or(Path::new(""));
        for line in text.lines() {
            let trimmed = line.trim();
            if let Some(include_path) = trimmed.strip_prefix("// @include ") {
                let included_path = include_base.join(include_path);
                let included = self.load_recursive(&included_path, visited)?;
                output.push_str(&included);
                output.push('\n');
            } else {
                output.push_str(line);
                output.push('\n');
            }
        }

        visited.remove(&canonical_key);
        Ok(output)
    }
}
