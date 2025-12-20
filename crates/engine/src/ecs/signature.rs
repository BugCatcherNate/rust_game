#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ComponentSignature(pub(crate) u16);

impl ComponentSignature {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn contains(self, kind: ComponentKind) -> bool {
        (self.0 & kind.bit()) != 0
    }

    pub const fn with(self, kind: ComponentKind) -> Self {
        Self(self.0 | kind.bit())
    }

    pub const fn without(self, kind: ComponentKind) -> Self {
        Self(self.0 & !kind.bit())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ComponentKind {
    Render,
    Input,
    Model,
    Camera,
    Light,
    Texture,
    Terrain,
    Script,
    Physics,
    Hierarchy,
    Attributes,
    ParticleEmitter,
    Particle,
}

impl ComponentKind {
    const fn bit(self) -> u16 {
        match self {
            ComponentKind::Render => 1 << 0,
            ComponentKind::Input => 1 << 1,
            ComponentKind::Model => 1 << 2,
            ComponentKind::Camera => 1 << 3,
            ComponentKind::Light => 1 << 4,
            ComponentKind::Texture => 1 << 5,
            ComponentKind::Terrain => 1 << 6,
            ComponentKind::Script => 1 << 7,
            ComponentKind::Physics => 1 << 8,
            ComponentKind::Hierarchy => 1 << 9,
            ComponentKind::Attributes => 1 << 10,
            ComponentKind::ParticleEmitter => 1 << 11,
            ComponentKind::Particle => 1 << 12,
        }
    }
}
