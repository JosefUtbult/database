pub trait DatabaseContent<Parameter, const PARAMETER_COUNT: usize>: Clone + Copy
where
    Parameter: Clone + Copy + Eq,
{
    fn set(&mut self, parameter: Parameter);
    fn get(&self, parameter: &Parameter) -> Parameter;
}
