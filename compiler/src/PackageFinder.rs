use crate::siko::{package::Package::PackageInfo, util};

pub struct Package {
    pub info: Option<PackageInfo>,
    pub files: Vec<String>,
}

impl Package {
    pub fn new() -> Self {
        Package {
            info: None,
            files: Vec::new(),
        }
    }

    pub fn addPath(&mut self, p: &std::path::Path) -> Vec<Package> {
        let inputPath = p;
        let mut subPackages = Vec::new();
        if inputPath.is_file() {
            self.files.push(format!("{}", inputPath.display()));
        } else if inputPath.is_dir() {
            let packageFilePath = inputPath.join("package.yaml");
            if packageFilePath.is_file() {
                let content = match std::fs::read_to_string(&packageFilePath) {
                    Ok(c) => c,
                    Err(err) => {
                        util::error(format!(
                            "Failed to read package file {}: {}",
                            packageFilePath.display(),
                            err
                        ));
                    }
                };
                let info = match PackageInfo::parseFromYaml(&content) {
                    Ok(info) => info,
                    Err(err) => {
                        util::error(format!(
                            "Failed to parse package file {}: {:?}",
                            packageFilePath.display(),
                            err
                        ));
                    }
                };
                let mut newPackage = Package {
                    info: Some(info),
                    files: Vec::new(),
                };
                for entry in std::fs::read_dir(inputPath).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_dir() {
                        let foundPackages = newPackage.addPath(&path);
                        subPackages.extend(foundPackages);
                    } else if let Some(extension) = path.extension() {
                        if extension == "sk" {
                            newPackage.files.push(format!("{}", path.display()));
                        }
                    }
                }
                subPackages.push(newPackage);
            } else {
                for entry in std::fs::read_dir(inputPath).unwrap() {
                    let entry = entry.unwrap();
                    let path = entry.path();
                    if path.is_dir() {
                        let foundPackages = self.addPath(&path);
                        subPackages.extend(foundPackages);
                    } else if let Some(extension) = path.extension() {
                        if extension == "sk" {
                            self.files.push(format!("{}", path.display()));
                        }
                    }
                }
            }
        }
        subPackages
    }
}

pub struct PackageFinder {
    pub packages: Vec<Package>,
}

impl PackageFinder {
    pub fn new() -> Self {
        PackageFinder { packages: Vec::new() }
    }

    pub fn processPaths(&mut self, p: Vec<String>) {
        let mut rootPackage = Package::new();
        for path in p {
            self.packages.extend(rootPackage.addPath(&std::path::Path::new(&path)));
        }
        self.packages.push(rootPackage);
    }

    pub fn dump(&self) {
        for p in &self.packages {
            if let Some(info) = &p.info {
                println!("Package: {} {}", info.name, info.version);
            } else {
                println!("Package: <root>");
            }
            for f in &p.files {
                println!("  File: {}", f);
            }
        }
    }
}
