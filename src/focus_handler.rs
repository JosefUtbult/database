pub trait FocusHandler<AbsKey, AbsField, FlatKey, FlatField> {
    fn get_focus_key(&self, key: FlatKey) -> AbsKey;
    fn get_focus_field(&self, field: FlatField) -> AbsField;
}
