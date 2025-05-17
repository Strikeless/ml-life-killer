use super::{Network, Value};

pub trait InputProvider<S> = Fn(&S) -> Value;

pub struct NetworkHarness<'a, S> {
    pub network: &'a mut Network,
    pub input_providers: Vec<Box<dyn InputProvider<S>>>,
}

impl<'a, S> NetworkHarness<'a, S> {
    pub fn new(network: &'a mut Network) -> Self {
        Self {
            network,
            input_providers: Vec::new(),
        }
    }

    pub fn with_inputs<I>(mut self, inputs: I) -> Self
    where
        I: IntoIterator<Item = Box<dyn InputProvider<S>>>,
    {
        self.add_inputs(inputs);
        self
    }

    pub fn add_inputs<I>(&mut self, inputs: I)
    where
        I: IntoIterator<Item = Box<dyn InputProvider<S>>>,
    {
        self.input_providers.extend(inputs);
    }

    pub fn with_input<F>(mut self, provider: F) -> Self
    where
        F: InputProvider<S> + 'static,
    {
        self.add_input(provider);
        self
    }

    pub fn add_input<F>(&mut self, provider: F)
    where
        F: InputProvider<S> + 'static,
    {
        self.add_boxed_input(Box::new(provider));
    }

    pub fn add_boxed_input(&mut self, provider: Box<dyn InputProvider<S>>) {
        self.input_providers.push(provider);
    }

    pub fn compute(&mut self, state: &S) -> impl Iterator<Item = Value> {
        self.update_inputs(state);

        self.network
            .compute()
            .into_iter()
    }

    fn update_inputs(&mut self, state: &S) {
        let input_values = self
            .input_providers
            .iter()
            .map(|input_provider| input_provider(state));

        self.network.input_layer.update(input_values);
    }
}
