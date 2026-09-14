// Issue 56 / LG-07 / LK-07: List cannot survive by moving behind a retained
// semantic facade or by acquiring a generic collection/storage compatibility shape.
use cordis_core::{
    Acquire, Collection, ResourceBag, Storage, acquire, collection, resource, storage,
};
use cordis_core::effect::{
    Acquire as EffectAcquire, Collection as EffectCollection, List as EffectList,
    ResourceBag as EffectResourceBag, Storage as EffectStorage,
};
use cordis_core::event::{
    Acquire as EventAcquire, Collection as EventCollection, List as EventList,
    ResourceBag as EventResourceBag, Storage as EventStorage,
};
use cordis_core::logger::{
    Acquire as LoggerAcquire, Collection as LoggerCollection, List as LoggerList,
    ResourceBag as LoggerResourceBag, Storage as LoggerStorage,
};
use cordis_core::service::{
    Acquire as ServiceAcquire, Collection as ServiceCollection, List as ServiceList,
    ResourceBag as ServiceResourceBag, Storage as ServiceStorage,
};

fn main() {}
