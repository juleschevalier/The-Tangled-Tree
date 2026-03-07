//! Diet categories used by creatures.

/// Feeding strategy encoded by a diet gene.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Diet {
    Herbivore,
    Omnivore,
    Carnivore,
}
