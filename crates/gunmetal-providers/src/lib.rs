use gunmetal_core::ProviderKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProviderClass {
    Subscription,
    Gateway,
    Direct,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderDefinition {
    pub kind: ProviderKind,
    pub class: ProviderClass,
    pub priority: usize,
}

pub fn builtin_providers() -> Vec<ProviderDefinition> {
    vec![
        ProviderDefinition {
            kind: ProviderKind::Codex,
            class: ProviderClass::Subscription,
            priority: 1,
        },
        ProviderDefinition {
            kind: ProviderKind::Copilot,
            class: ProviderClass::Subscription,
            priority: 2,
        },
        ProviderDefinition {
            kind: ProviderKind::OpenRouter,
            class: ProviderClass::Gateway,
            priority: 3,
        },
        ProviderDefinition {
            kind: ProviderKind::Zen,
            class: ProviderClass::Gateway,
            priority: 4,
        },
        ProviderDefinition {
            kind: ProviderKind::OpenAi,
            class: ProviderClass::Direct,
            priority: 5,
        },
        ProviderDefinition {
            kind: ProviderKind::Azure,
            class: ProviderClass::Direct,
            priority: 6,
        },
        ProviderDefinition {
            kind: ProviderKind::Nvidia,
            class: ProviderClass::Direct,
            priority: 7,
        },
    ]
}

#[cfg(test)]
mod tests {
    use gunmetal_core::ProviderKind;

    use super::{ProviderClass, builtin_providers};

    #[test]
    fn builtin_provider_order_matches_product_priority() {
        let providers = builtin_providers();
        assert_eq!(providers[0].kind, ProviderKind::Codex);
        assert_eq!(providers[1].kind, ProviderKind::Copilot);
        assert_eq!(providers[2].kind, ProviderKind::OpenRouter);
        assert_eq!(providers[3].kind, ProviderKind::Zen);
    }

    #[test]
    fn provider_classes_are_partitioned() {
        let providers = builtin_providers();
        assert_eq!(providers[0].class, ProviderClass::Subscription);
        assert_eq!(providers[2].class, ProviderClass::Gateway);
        assert_eq!(providers[4].class, ProviderClass::Direct);
    }
}
