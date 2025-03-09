use crate::network::Network;

pub trait TrainerAdapterFactory<T>
where
    T: TrainerAdapter,
{
    fn create_adapter(&self) -> T;
}

pub trait TrainerAdapter: Sync {
    fn try_out(&self, network: &mut Network) -> f32;
}
