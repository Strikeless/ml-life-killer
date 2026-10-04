#[macro_export]
macro_rules! new_position_type {
    {
        $($type_visibility:vis struct $type_name:ident);*;
    } => {
        $(
            #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
            $type_visibility struct $type_name([u32; 2]);

            impl crate::position::LifePositionTrait for $type_name {
                fn new(x: u32, y: u32) -> Self {
                    Self([x, y])
                }

                fn x(&self) -> u32 {
                    self.0[0]
                }

                fn y(&self) -> u32 {
                    self.0[1]
                }

                fn with_x(&self, x: u32) -> Self {
                    Self([x, self.y()])
                }

                fn with_y(&self, y: u32) -> Self {
                    Self([self.x(), y])
                }
            }

            impl From<[u32; 2]> for $type_name {
                fn from([x, y]: [u32; 2]) -> Self {
                    Self([x, y])
                }
            }
        )*
    };
}

pub trait LifePositionTrait {
    fn new(x: u32, y: u32) -> Self;

    fn x(&self) -> u32;
    fn y(&self) -> u32;

    fn with_x(&self, x: u32) -> Self;
    fn with_y(&self, y: u32) -> Self;

    fn map<O: LifePositionTrait>(&self, map_fn: fn(axis: u32) -> u32) -> O {
        let x = map_fn(self.x());
        let y = map_fn(self.y());
        O::new(x, y)
    }

    fn reduce<I: LifePositionTrait, O: LifePositionTrait>(&self, other: &I, zip_fn: fn(this: u32, other: u32) -> u32) -> O {
        let x = zip_fn(self.x(), other.x());
        let y = zip_fn(self.y(), other.y());
        O::new(x, y)
    }
}
