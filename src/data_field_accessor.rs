pub enum AccessorError {
    TypeMissmatch(&'static str),
}

pub trait DataFieldAccessor<AbsKey, AbsField> {
    fn get(&self, key: AbsKey) -> AbsField;
    fn set(&mut self, field: AbsField);
}

pub trait DataFieldTryAccessor<AbsKey, T> {
    fn try_get(&self, key: AbsKey) -> Result<T, AccessorError>;
    fn try_set(&mut self, key: AbsKey, value: T) -> Result<(), AccessorError>;
}

pub trait FolderAccessor<T, AbsFolder> {
    fn try_get(&self, folder: AbsFolder) -> Result<&T, AccessorError>;
    fn try_get_mut(&mut self, folder: AbsFolder) -> Result<&mut T, AccessorError>;
}
