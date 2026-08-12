use super::domain::CategoryKind;

#[derive(Debug, Clone, Copy)]
pub(crate) struct InitialCategory {
    name: &'static str,
    kind: CategoryKind,
}

impl InitialCategory {
    const fn income(name: &'static str) -> Self {
        Self {
            name,
            kind: CategoryKind::Income,
        }
    }

    const fn expense(name: &'static str) -> Self {
        Self {
            name,
            kind: CategoryKind::Expense,
        }
    }

    pub(crate) const fn name(self) -> &'static str {
        self.name
    }

    pub(crate) const fn kind(self) -> &'static str {
        self.kind.as_str()
    }
}

pub(crate) const INITIAL_CATEGORIES: &[InitialCategory] = &[
    InitialCategory::income("Salaire"),
    InitialCategory::income("Revenus professionnels"),
    InitialCategory::income("Revenus locatifs"),
    InitialCategory::income("Intérêts et dividendes"),
    InitialCategory::income("Prestations et pensions"),
    InitialCategory::income("Autres revenus"),
    InitialCategory::expense("Logement"),
    InitialCategory::expense("Alimentation"),
    InitialCategory::expense("Restaurants"),
    InitialCategory::expense("Transport"),
    InitialCategory::expense("Santé"),
    InitialCategory::expense("Assurances"),
    InitialCategory::expense("Abonnements"),
    InitialCategory::expense("Loisirs"),
    InitialCategory::expense("Achats personnels"),
    InitialCategory::expense("Voyages"),
    InitialCategory::expense("Impôts et taxes"),
    InitialCategory::expense("Frais bancaires"),
    InitialCategory::expense("Autres dépenses"),
];

#[cfg(test)]
mod tests {
    use super::INITIAL_CATEGORIES;

    #[test]
    fn initial_catalog_has_expected_distribution() {
        assert_eq!(INITIAL_CATEGORIES.len(), 19);
        assert_eq!(
            INITIAL_CATEGORIES
                .iter()
                .filter(|category| category.kind() == "income")
                .count(),
            6
        );
        assert_eq!(
            INITIAL_CATEGORIES
                .iter()
                .filter(|category| category.kind() == "expense")
                .count(),
            13
        );
    }
}
