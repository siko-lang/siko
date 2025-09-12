use crate::siko::miniyaml;

pub struct Dependency {
    pub name: String,
    pub version: String,
}
pub struct PackageInfo {
    pub name: String,
    pub version: String,
    pub dependencies: Vec<Dependency>,
}

#[derive(Debug)]
pub enum Error {
    YamlError(miniyaml::Error),
    PackageNotFound,
    NameNotFound,
    VersionNotFound,
    DependencyNotAnObject,
    DependencyNameNotFound,
    DependencyVersionNotFound,
}

impl From<miniyaml::Error> for Error {
    fn from(err: miniyaml::Error) -> Self {
        Error::YamlError(err)
    }
}

impl PackageInfo {
    pub fn new(name: String, version: String, dependencies: Vec<Dependency>) -> Self {
        Self {
            name,
            version,
            dependencies,
        }
    }

    pub fn parseFromYaml(yaml: &str) -> Result<Self, Error> {
        let mut parser = miniyaml::Parser::new(yaml);
        let yamlValue = parser.parseValue(0)?;
        if let miniyaml::YamlValue::Object(object) = yamlValue {
            let name = if let Some(miniyaml::YamlValue::String(name)) = object.get("name") {
                name.clone()
            } else {
                return Err(Error::NameNotFound);
            };
            let version = if let Some(miniyaml::YamlValue::String(version)) = object.get("version") {
                version.clone()
            } else {
                return Err(Error::VersionNotFound);
            };
            let dependencies = if let Some(miniyaml::YamlValue::List(deps)) = object.get("dependencies") {
                let mut result = Vec::new();
                for dep in deps {
                    if let miniyaml::YamlValue::Object(depObj) = dep {
                        let name = if let Some(miniyaml::YamlValue::String(name)) = depObj.get("name") {
                            name.clone()
                        } else {
                            return Err(Error::DependencyNameNotFound);
                        };
                        let version = if let Some(miniyaml::YamlValue::String(version)) = depObj.get("version") {
                            version.clone()
                        } else {
                            return Err(Error::DependencyVersionNotFound);
                        };
                        result.push(Dependency { name, version });
                    } else {
                        return Err(Error::DependencyNotAnObject);
                    }
                }
                result
            } else {
                Vec::new()
            };
            Ok(PackageInfo::new(name, version, dependencies))
        } else {
            Err(Error::PackageNotFound)
        }
    }
}
