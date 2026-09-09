use crate::contract::Dimensions;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProfileDescriptor {
    pub name: &'static str,
    pub version: &'static str,
    pub dimensions: DimensionsDescriptor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DimensionsDescriptor {
    pub decomposition: &'static str,
    pub internal_structure: &'static str,
    pub dependency_policy: &'static str,
    pub visibility: &'static str,
}

impl ProfileDescriptor {
    pub fn dimensions(self) -> Dimensions {
        Dimensions {
            decomposition: self.dimensions.decomposition.to_owned(),
            internal_structure: self.dimensions.internal_structure.to_owned(),
            dependency_policy: self.dimensions.dependency_policy.to_owned(),
            visibility: self.dimensions.visibility.to_owned(),
        }
    }
}

pub fn resolve_profile(name: &str, version: &str) -> Option<ProfileDescriptor> {
    match (name, version) {
        ("angular", "0.1") => Some(ProfileDescriptor {
            name: "angular",
            version: "0.1",
            dimensions: DimensionsDescriptor {
                decomposition: "feature-areas",
                internal_structure: "angular-style-guide",
                dependency_policy: "framework-unspecified",
                visibility: "framework-unspecified",
            },
        }),
        _ => None,
    }
}
