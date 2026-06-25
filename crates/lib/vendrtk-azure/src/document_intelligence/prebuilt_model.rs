pub enum PrebuiltModel {
    Layout,
    Invoice,
    Contract
}

impl AsRef<str> for PrebuiltModel {
    fn as_ref(&self) -> &str {
        match self {
            PrebuiltModel::Layout => "prebuilt-layout",
            PrebuiltModel::Invoice => "prebuilt-invoice",
            PrebuiltModel::Contract => "prebuilt-contract",
        }
    }
}