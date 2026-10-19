//! Names of the constraints Canyon creates: the same on every backend.

use crate::model::{Ident, MAX_IDENT_LEN, QualifiedName};

pub fn pk_name(table: &QualifiedName) -> Ident {
    fit(format!("{}_pkey", table.name))
}

pub fn fk_name(table: &QualifiedName, columns: &[Ident]) -> Ident {
    let cols: Vec<&str> = columns.iter().map(Ident::as_str).collect();
    fit(format!("{}_{}_fkey", table.name, cols.join("_")))
}

// Over 63 bytes: keep 54, then `_` and 8 hex of the full name's hash
fn fit(name: String) -> Ident {
    if name.len() <= MAX_IDENT_LEN {
        return Ident::from_catalog(&name);
    }
    let hash = fnv1a32(&name);
    // Catalog names may be non-ASCII: cut on a char boundary
    let mut cut = MAX_IDENT_LEN - 9;
    while !name.is_char_boundary(cut) {
        cut -= 1;
    }
    Ident::from_catalog(&format!("{}_{hash:08x}", &name[..cut]))
}

/// FNV-1a 32: stable across Rust versions, unlike the std hashers.
fn fnv1a32(text: &str) -> u32 {
    text.bytes().fold(0x811c_9dc5, |hash, byte| {
        (hash ^ u32::from(byte)).wrapping_mul(0x0100_0193)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn short_names() {
        let t = QualifiedName::parse("player").unwrap();
        assert_eq!(pk_name(&t).as_str(), "player_pkey");
        assert_eq!(
            fk_name(&t, &[Ident::new("league").unwrap()]).as_str(),
            "player_league_fkey"
        );
    }

    #[test]
    fn long_names_are_cut_with_a_stable_hash() {
        let t = QualifiedName::parse(&"t".repeat(60)).unwrap();
        let a = pk_name(&t);
        assert_eq!(a.as_str().len(), 63);
        assert_eq!(a, pk_name(&t), "same input, same name");
        let other = QualifiedName::parse(&format!("{}x", "t".repeat(59))).unwrap();
        assert_ne!(a, pk_name(&other));
    }

    #[test]
    fn fnv1a_reference_values() {
        assert_eq!(fnv1a32(""), 0x811c9dc5);
        assert_eq!(fnv1a32("a"), 0xe40c292c);
    }

    #[test]
    fn long_non_ascii_names_are_cut_on_a_char_boundary() {
        // "a" + 40 × "é": byte 54 falls inside an "é", so the cut moves back to 53
        let t = QualifiedName {
            schema: None,
            name: Ident::from_catalog(&format!("a{}", "é".repeat(40))),
        };
        let name = pk_name(&t);
        assert_eq!(name.as_str().len(), 53 + 9);
        assert!(name.as_str().is_char_boundary(53));
    }

    #[test]
    fn long_foreign_key_names_are_cut_too() {
        let t = QualifiedName::parse(&"t".repeat(40)).unwrap();
        let fk = fk_name(&t, &[Ident::new(&"c".repeat(30)).unwrap()]);
        assert_eq!(fk.as_str().len(), 63);
        assert_eq!(fk, fk_name(&t, &[Ident::new(&"c".repeat(30)).unwrap()]));
    }
}
