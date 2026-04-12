//! Place projections for MIR

use super::{LocalVar, Place, Projection};

impl Place {
    /// Create a field projection: `place.field`
    pub fn field(&self, field: usize) -> Place {
        let mut p = self.clone();
        p.projection.push(Projection::Field(field));
        p
    }

    /// Create a deref projection: `*place`
    pub fn deref(&self) -> Place {
        let mut p = self.clone();
        p.projection.push(Projection::Deref);
        p
    }

    /// Create an index projection: `place[index]`
    pub fn index(&self, index: Place) -> Place {
        let mut p = self.clone();
        p.projection.push(Projection::Index(index));
        p
    }

    /// Create a slice projection: `place[start..end]`
    pub fn subslice(&self, start: usize, end: usize) -> Place {
        let mut p = self.clone();
        p.projection.push(Projection::Subslice { start, end });
        p
    }

    /// Get the base local var without projections
    pub fn base(&self) -> LocalVar {
        self.local
    }

    /// Check if this place is a simple local var (no projections)
    pub fn is_simple(&self) -> bool {
        self.projection.is_empty()
    }

    /// Get all field indices used in projections
    pub fn field_paths(&self) -> Vec<usize> {
        self.projection
            .iter()
            .filter_map(|p| {
                if let Projection::Field(i) = p {
                    Some(*i)
                } else {
                    None
                }
            })
            .collect()
    }

    /// Check if this place potentially overlaps with another
    pub fn may_overlap(&self, other: &Place) -> bool {
        if self.base() != other.base() {
            return false;
        }

        let self_fields = self.field_paths();
        let other_fields = other.field_paths();

        for sf in &self_fields {
            if other_fields.contains(sf) {
                return true;
            }
        }

        self_fields.is_empty() || other_fields.is_empty()
    }
}
