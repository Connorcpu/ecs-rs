
//! Types to process the world and entities.

pub use self::entity::{EntitySystem, EntityProcess};
pub use self::interact::{InteractSystem, InteractProcess};
pub use self::interval::{IntervalSystem};

use EntityData;
use ComponentManager;
use DataHelper;

pub mod entity;
pub mod interact;
pub mod interval;

/// Generic base system type.
pub trait System: 'static
{
    type Components: ComponentManager;
    /// Optional method called when an entity is activated.
    fn activated(&mut self, _: &EntityData, _: &Self::Components)
    {

    }

    /// Optional method called when an entity is reactivated.
    ///
    /// By default it calls deactivated() followed by activated()
    fn reactivated(&mut self, e: &EntityData, c: &Self::Components)
    {
        self.deactivated(e, c);
        self.activated(e, c);
    }

    /// Optional method called when an entity is deactivated.
    fn deactivated(&mut self, _: &EntityData, _: &Self::Components)
    {

    }

    fn is_active(&self) -> bool
    {
        true
    }
}

pub trait Process: System
{
    /// Process the world.
    fn process(&mut self, &mut DataHelper<<Self as System>::Components>);
}
