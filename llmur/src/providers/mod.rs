pub mod openai;
pub mod azure;

pub(crate) mod utils;

// region:    --- Transformer
pub struct Transformation<Result, Loss> {
    pub result: Result,
    pub loss: Loss,
}
pub trait Transformer<Result, Context, Loss>: Sized
where
    Context: TransformationContext<Self, Result>,
    Loss: TransformationLoss<Self, Result>,
{
    fn transform(self, context: Context) -> Transformation<Result, Loss>;
}

pub trait TransformationContext<From, To> {}
pub trait TransformationLoss<From, To> {}
// endregion: --- Transformer

pub trait ExposesDeployment {
    fn get_deployment_ref(&self) -> &str;
}

pub trait ExposesUsage {
    fn get_input_tokens(&self) -> u64;
    fn get_output_tokens(&self) -> u64;
}