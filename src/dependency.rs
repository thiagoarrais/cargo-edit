use toml_edit;

#[derive(Debug, Hash, PartialEq, Eq, Clone)]
enum DependencySource {
    Version {
        version: Option<String>,
        path: Option<String>,
    },
    Git(String),
}

/// A dependency handled by Cargo
#[derive(Debug, Hash, PartialEq, Eq, Clone)]
pub struct Dependency {
    /// The name of the dependency (as it is set in its `Cargo.toml` and known to crates.io)
    pub name: String,
    optional: bool,
    default_features: bool,
    source: DependencySource,
}

impl Default for Dependency {
    fn default() -> Dependency {
        Dependency {
            name: "".into(),
            optional: false,
            default_features: true,
            source: DependencySource::Version {
                version: None,
                path: None,
            },
        }
    }
}

impl Dependency {
    /// Create a new dependency with a name
    pub fn new(name: &str) -> Dependency {
        Dependency {
            name: name.into(),
            ..Dependency::default()
        }
    }

    /// Set dependency to a given version
    pub fn set_version(mut self, version: &str) -> Dependency {
        // versions might have semver metadata appended which we do not want to
        // store in the cargo toml files.  This would cause a warning upon compilation
        // ("version requirement […] includes semver metadata which will be ignored")
        let version = version.split('+').next().unwrap();
        let old_source = self.source;
        let old_path = match old_source {
            DependencySource::Version { path, .. } => path,
            _ => None,
        };
        self.source = DependencySource::Version {
            version: Some(version.into()),
            path: old_path,
        };
        self
    }

    /// Set dependency to a given repository
    pub fn set_git(mut self, repo: &str) -> Dependency {
        self.source = DependencySource::Git(repo.into());
        self
    }

    /// Set dependency to a given path
    pub fn set_path(mut self, path: &str) -> Dependency {
        let old_source = self.source;
        let old_version = match old_source {
            DependencySource::Version { version, .. } => version,
            _ => None,
        };
        self.source = DependencySource::Version {
            version: old_version,
            path: Some(path.into()),
        };
        self
    }

    /// Set whether the dependency is optional
    pub fn set_optional(mut self, opt: bool) -> Dependency {
        self.optional = opt;
        self
    }

    /// Set the value of default-features for the dependency
    pub fn set_default_features(mut self, default_features: bool) -> Dependency {
        self.default_features = default_features;
        self
    }

    /// Get version of dependency
    pub fn version(&self) -> Option<&str> {
        if let DependencySource::Version {
            version: Some(ref version),
            ..
        } = self.source
        {
            Some(version)
        } else {
            None
        }
    }

    /// Convert dependency to TOML
    ///
    /// Returns a tuple with the dependency's name and either the version as a `String`
    /// or the path/git repository as an `InlineTable`.
    /// (If the dependency is set as `optional` or `default-features` is set to `false`,
    /// an `InlineTable` is returned in any case.)
    pub fn to_toml(&self) -> (String, toml_edit::Item) {
        let data: toml_edit::Item =
            match (self.optional, self.default_features, self.source.clone()) {
                // Extra short when version flag only
                (
                    false,
                    true,
                    DependencySource::Version {
                        version: Some(v),
                        path: None,
                    },
                ) => toml_edit::value(v),
                // Other cases are represented as an inline table
                (optional, default_features, source) => {
                    let mut data = toml_edit::InlineTable::default();

                    match source {
                        DependencySource::Version { version, path } => {
                            if let Some(v) = version {
                                data.get_or_insert("version", v);
                            }
                            if let Some(p) = path {
                                data.get_or_insert("path", p);
                            }
                        }
                        DependencySource::Git(v) => {
                            data.get_or_insert("git", v);
                        }
                    }
                    if self.optional {
                        data.get_or_insert("optional", optional);
                    }
                    if !self.default_features {
                        data.get_or_insert("default-features", default_features);
                    }

                    data.fmt();
                    toml_edit::value(toml_edit::Value::InlineTable(data))
                }
            };

        (self.name.clone(), data)
    }
}
