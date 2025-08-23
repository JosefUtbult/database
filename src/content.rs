/// The `DatabaseContent` is a structure containing all parameters in the database. This structure
/// is made by the user and then expanded upon using the `Database` proc-macro
pub trait DatabaseContent<Parameter, const PARAMETER_COUNT: usize>: Clone + Copy
where
    Parameter: Clone + Copy + Eq,
{
    /// Set the value of a parameter in the context. This function should be automatically created
    /// by the `Database` proc-macro.
    fn set(&mut self, parameter: Parameter);

    /// Get the value of a parameter in the context. This function should be automatically created
    /// by the `Database` proc-macro. The user will presume that the parameter returned is of the
    /// same type that the one requested. All other parameters will cause the program to panic.
    fn get(&self, parameter: &Parameter) -> Parameter;
}
