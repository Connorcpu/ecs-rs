
use std::ops::{Deref, DerefMut};

use {BuildData, EntityData, ModifyData};
use {Entity, EntityIter, EntityBuilder, EntityModifier};
use {System};
use entity::EntityManager;

enum Event<'a, T> where T: ComponentManager
{
    BuildEntity(Entity, Box<EntityBuilder<T>+'a>),
    ModifyEntity(Entity, Box<EntityModifier<T>+'a>),
    RemoveEntity(Entity),
}

pub struct World<T, U> where T: ComponentManager, U: SystemManager<Components=T>
{
    pub systems: U,
    pub data: DataHelper<T>,
}

pub struct DataHelper<T> where T: ComponentManager
{
    pub components: T,
    entities: EntityManager,
    event_queue: Vec<Event<'static, T>>,
}

pub unsafe trait ComponentManager: 'static
{
    unsafe fn new() -> Self;
    unsafe fn remove_all(&mut self, en: &Entity);
}

pub unsafe trait SystemManager: 'static
{
    type Components: ComponentManager;
    unsafe fn new() -> Self;
    unsafe fn activated(&mut self, en: EntityData, co: &Self::Components);
    unsafe fn reactivated(&mut self, en: EntityData, co: &Self::Components);
    unsafe fn deactivated(&mut self, en: EntityData, co: &Self::Components);
    unsafe fn update(&mut self, co: &mut DataHelper<Self::Components>);
}

impl<T: ComponentManager, U: SystemManager<Components=T>> Deref for World<T, U>
{
    type Target = DataHelper<T>;
    fn deref(&self) -> &DataHelper<T>
    {
        &self.data
    }
}

impl<T: ComponentManager, U: SystemManager<Components=T>> DerefMut for World<T, U>
{
    fn deref_mut(&mut self) -> &mut DataHelper<T>
    {
        &mut self.data
    }
}

impl<T: ComponentManager> Deref for DataHelper<T>
{
    type Target = T;
    fn deref(&self) -> &T
    {
        &self.components
    }
}

impl<T: ComponentManager> DerefMut for DataHelper<T>
{
    fn deref_mut(&mut self) -> &mut T
    {
        &mut self.components
    }
}

impl<T: ComponentManager> DataHelper<T>
{
    pub fn with_entity_data<F, R>(&mut self, entity: &Entity, mut call: F) -> Option<R>
        where F: FnMut(EntityData, &mut T) -> R
    {
        if self.entities.is_valid(entity) {
            Some(call(EntityData(entity), self))
        } else {
            None
        }
    }

    pub fn create_entity<B>(&mut self, builder: B) -> Entity where B: EntityBuilder<T>+'static
    {
        let entity = self.entities.create();
        self.event_queue.push(Event::BuildEntity(entity, Box::new(builder)));
        entity
    }

    pub fn modify_entity<M>(&mut self, entity: Entity, modifier: M) where M: EntityModifier<T>+'static
    {
        self.event_queue.push(Event::ModifyEntity(entity, Box::new(modifier)));
    }

    pub fn remove_entity(&mut self, entity: Entity)
    {
        self.event_queue.push(Event::RemoveEntity(entity));
    }
}

impl<T: ComponentManager, U: SystemManager<Components=T>> World<T, U>
{
    pub fn new() -> World<T, U>
    {
        World {
            systems: unsafe { <U as SystemManager>::new() },
            data: DataHelper {
                components: unsafe { <T as ComponentManager>::new() },
                entities: EntityManager::new(),
                event_queue: Vec::new(),
            },
        }
    }

    pub fn create_entity<B>(&mut self, mut builder: B) -> Entity where B: EntityBuilder<T>
    {
        let entity = self.data.entities.create();
        builder.build(BuildData(&entity), &mut self.data.components);
        unsafe { self.systems.activated(EntityData(&entity), &self.data.components); }
        entity
    }

    pub fn with_entity_data<F, R>(&mut self, entity: &Entity, mut call: F) -> Option<R>
        where F: FnMut(EntityData, &mut T) -> R
    {
        if self.data.entities.is_valid(entity) {
            Some(call(EntityData(entity), &mut self.data))
        } else {
            None
        }
    }

    pub fn entities(&self) -> EntityIter<T>
    {
        self.data.entities.iter()
    }

    pub fn modify_entity<M>(&mut self, entity: Entity, mut modifier: M) where M: EntityModifier<T>
    {
        modifier.modify(ModifyData(&entity), &mut self.data.components);
        unsafe { self.systems.reactivated(EntityData(&entity), &self.data.components); }
    }

    pub fn remove_entity(&mut self, entity: Entity)
    {
        self.process_event(Event::RemoveEntity(entity));
    }

    fn process_event(&mut self, event: Event<T>)
    {
        process_event(&mut self.data.components, &mut self.systems, &mut self.data.entities, event);
    }

    fn flush_queue(&mut self)
    {
        for event in self.data.event_queue.drain()
        {
            process_event(&mut self.data.components, &mut self.systems, &mut self.data.entities, event);
        }
    }

    pub fn update(&mut self)
    {
        self.flush_queue();
        unsafe { self.systems.update(&mut self.data); }
    }
}

// This function has to be external to World because of borrowing rules
fn process_event<T: ComponentManager, U: SystemManager<Components=T>>(components: &mut T, systems: &mut U, entities: &mut EntityManager, event: Event<T>)
{
    match event
    {
        Event::BuildEntity(entity, mut builder) => {
            builder.build(BuildData(&entity), components);
            unsafe { systems.activated(EntityData(&entity), components); }
        },
        Event::ModifyEntity(entity, mut modifier) => {
            modifier.modify(ModifyData(&entity), components);
            unsafe { systems.reactivated(EntityData(&entity), components); }
        },
        Event::RemoveEntity(entity) => {
            unsafe {
                systems.deactivated(EntityData(&entity), components);
                components.remove_all(&entity);
            }
            entities.remove(&entity);
        }
    }
}
