use crate::siko::{
    backend::borrowcheck::{
        functionprofiles::FunctionProfile::{FunctionProfile, Link},
        DataGroups::{EnumDef, ExtendedType, FieldDef, StructDef, VariantDef},
    },
    hir::{Apply::Apply, Substitution::Substitution},
};

impl Apply for ExtendedType {
    fn apply(self, sub: &Substitution) -> Self {
        let newVars = self.vars.clone().apply(sub);
        ExtendedType {
            ty: self.ty.clone(),
            vars: newVars,
        }
    }
}

impl Apply for FieldDef {
    fn apply(self, sub: &Substitution) -> Self {
        FieldDef {
            name: self.name.clone(),
            ty: self.ty.clone().apply(sub),
            inGroup: self.inGroup,
        }
    }
}

impl Apply for StructDef {
    fn apply(self, sub: &Substitution) -> Self {
        let newTy = self.ty.clone().apply(sub);
        let newFields = self.fields.clone().apply(sub);
        StructDef {
            name: self.name.clone(),
            ty: newTy,
            fields: newFields,
        }
    }
}

impl Apply for VariantDef {
    fn apply(self, sub: &Substitution) -> Self {
        VariantDef {
            name: self.name.clone(),
            ty: self.ty.clone().apply(sub),
            inGroup: self.inGroup,
        }
    }
}

impl Apply for EnumDef {
    fn apply(self, sub: &Substitution) -> Self {
        let newTy = self.ty.clone().apply(sub);
        let newVariants = self.variants.clone().apply(sub);
        EnumDef {
            name: self.name.clone(),
            ty: newTy,
            variants: newVariants,
        }
    }
}

impl Apply for FunctionProfile {
    fn apply(self, sub: &Substitution) -> Self {
        let newArgs = self.args.clone().apply(sub);
        let newResult = self.result.clone().apply(sub);
        let newLinks = self.links.clone().apply(sub);
        FunctionProfile {
            name: self.name.clone(),
            args: newArgs,
            result: newResult,
            links: newLinks,
        }
    }
}

impl Apply for Link {
    fn apply(self, sub: &Substitution) -> Self {
        Link {
            from: self.from.clone().apply(sub),
            to: self.to.clone().apply(sub),
        }
    }
}
