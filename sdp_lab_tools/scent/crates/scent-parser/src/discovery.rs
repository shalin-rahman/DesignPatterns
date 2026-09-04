//! Deterministic, non-MSBuild C# project discovery.

use std::{
    collections::BTreeSet,
    fs, io,
    path::{Path, PathBuf},
};

use scent_domain::NormalizedPath;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryOptions {
    pub excluded_directory_names: BTreeSet<String>,
    pub excluded_file_suffixes: BTreeSet<String>,
}

impl Default for DiscoveryOptions {
    fn default() -> Self {
        Self {
            excluded_directory_names: ["bin", "obj", "generated"].map(String::from).into(),
            excluded_file_suffixes: [".g.cs", ".Designer.cs"].map(String::from).into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectDiscoveryStatus {
    Discovered,
    PartiallyResolved,
    Unresolved,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiscoveryResult {
    pub status: ProjectDiscoveryStatus,
    pub solution_files: Vec<NormalizedPath>,
    pub project_files: Vec<NormalizedPath>,
    pub source_files: Vec<NormalizedPath>,
    pub excluded_source_files: Vec<NormalizedPath>,
}

#[derive(Debug)]
pub enum DiscoveryError {
    ReadDirectory { path: PathBuf, source: io::Error },
    ReadEntry { path: PathBuf, source: io::Error },
    InvalidRelativePath { path: PathBuf },
}

/// Discovers C# artifacts beneath `root` without evaluating `MSBuild` files.
///
/// # Errors
///
/// Returns an error when the root cannot be read or an encountered filesystem
/// entry cannot be inspected.
pub fn discover_csharp(
    root: &Path,
    options: &DiscoveryOptions,
) -> Result<DiscoveryResult, DiscoveryError> {
    let mut result = DiscoveryResult {
        status: ProjectDiscoveryStatus::Unresolved,
        solution_files: Vec::new(),
        project_files: Vec::new(),
        source_files: Vec::new(),
        excluded_source_files: Vec::new(),
    };
    visit_directory(root, root, options, &mut result)?;
    result.status = status_for(&result);
    Ok(result)
}

fn visit_directory(
    root: &Path,
    current: &Path,
    options: &DiscoveryOptions,
    result: &mut DiscoveryResult,
) -> Result<(), DiscoveryError> {
    let mut entries = fs::read_dir(current)
        .map_err(|source| DiscoveryError::ReadDirectory {
            path: current.into(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| DiscoveryError::ReadEntry {
            path: current.into(),
            source,
        })?;
    entries.sort_by_key(fs::DirEntry::file_name);

    for entry in entries {
        let path = entry.path();
        let file_type = entry
            .file_type()
            .map_err(|source| DiscoveryError::ReadEntry {
                path: path.clone(),
                source,
            })?;
        if file_type.is_symlink() {
            continue;
        }
        if file_type.is_dir() {
            if is_excluded_directory(&path, options) {
                continue;
            }
            visit_directory(root, &path, options, result)?;
        } else if file_type.is_file() {
            classify_file(root, &path, options, result)?;
        }
    }
    Ok(())
}

fn classify_file(
    root: &Path,
    path: &Path,
    options: &DiscoveryOptions,
    result: &mut DiscoveryResult,
) -> Result<(), DiscoveryError> {
    let relative = relative_path(root, path)?;
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    match path.extension().and_then(|extension| extension.to_str()) {
        Some("sln" | "slnx") => result.solution_files.push(relative),
        Some("csproj") => result.project_files.push(relative),
        Some("cs") if is_excluded_source(name, options) => {
            result.excluded_source_files.push(relative);
        }
        Some("cs") => result.source_files.push(relative),
        _ => {}
    }
    Ok(())
}

fn relative_path(root: &Path, path: &Path) -> Result<NormalizedPath, DiscoveryError> {
    let relative = path
        .strip_prefix(root)
        .map_err(|_| DiscoveryError::InvalidRelativePath { path: path.into() })?;
    let display = relative.to_string_lossy();
    NormalizedPath::parse(&display)
        .map_err(|_| DiscoveryError::InvalidRelativePath { path: path.into() })
}

fn is_excluded_directory(path: &Path, options: &DiscoveryOptions) -> bool {
    path.file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| options.excluded_directory_names.contains(name))
}

fn is_excluded_source(name: &str, options: &DiscoveryOptions) -> bool {
    options
        .excluded_file_suffixes
        .iter()
        .any(|suffix| name.ends_with(suffix))
}

fn status_for(result: &DiscoveryResult) -> ProjectDiscoveryStatus {
    if !result.project_files.is_empty() {
        ProjectDiscoveryStatus::Discovered
    } else if !result.source_files.is_empty() || !result.solution_files.is_empty() {
        ProjectDiscoveryStatus::PartiallyResolved
    } else {
        ProjectDiscoveryStatus::Unresolved
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use scent_domain::NormalizedPath;

    use super::{discover_csharp, DiscoveryOptions, ProjectDiscoveryStatus};

    #[test]
    fn discovers_and_sorts_project_artifacts_without_generated_sources() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/discovery");
        let result = discover_csharp(&root, &DiscoveryOptions::default()).unwrap();

        assert_eq!(result.status, ProjectDiscoveryStatus::Discovered);
        assert_eq!(
            result
                .solution_files
                .iter()
                .map(NormalizedPath::as_str)
                .collect::<Vec<_>>(),
            ["Demo.sln"]
        );
        assert_eq!(
            result
                .project_files
                .iter()
                .map(NormalizedPath::as_str)
                .collect::<Vec<_>>(),
            ["Demo.csproj"]
        );
        assert_eq!(
            result
                .source_files
                .iter()
                .map(NormalizedPath::as_str)
                .collect::<Vec<_>>(),
            ["src/App.cs"]
        );
        assert_eq!(
            result
                .excluded_source_files
                .iter()
                .map(NormalizedPath::as_str)
                .collect::<Vec<_>>(),
            ["src/App.g.cs"]
        );
    }
}
