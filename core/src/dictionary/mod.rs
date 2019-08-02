//! This module contains the concept of a DICOM data dictionary.
//!
//! The standard data dictionary is available in the `dicom-std-dict` crate.

pub mod stub;

use crate::header::{Tag, VR};
use std::fmt::Debug;
use std::str::FromStr;

/// Specification of a range of tags pertaining to an attribute.
/// Very often, the dictionary of attributes indicates a unique `(group,elem)`
/// for a specific attribute, but occasionally a range of groups or elements
/// is indicated instead (e.g. _Pixel Data_ is associated with ).
#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq)]
pub enum TagRange {
    /// Only a specific tag
    Single(Tag),
    /// The two rightmost digits of the _group_ portion are open:
    /// `(GGxx,EEEE)`
    Group100(Tag),
    /// The two rightmost digits of the _element_ portion are open:
    /// `(GGGG,EExx)`
    Element100(Tag),
}

impl TagRange {
    /// Retrieve the inner tag representation of this range.
    pub fn inner(self) -> Tag {
        match self {
            TagRange::Single(inner) => inner, 
            TagRange::Group100(inner) => inner,
            TagRange::Element100(inner) => inner,
        }
    }

    /// Check whether this range contains the given tag.
    pub fn contains(self, tag: Tag) -> bool {
        match self {
            TagRange::Single(inner) => inner == tag,
            TagRange::Group100(inner) => inner.group() >> 8 == tag.group() >> 8 && inner.element() == tag.element(),
            TagRange::Element100(inner) => inner.group() == tag.group() && inner.element() >> 8 == tag.element() >> 8,
        }
    }
}

/// An error returned when parsing an invalid tag range.
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq)]
pub struct TagRangeParseError(&'static str);

impl FromStr for TagRange {
    type Err = TagRangeParseError;

    fn from_str(mut s: &str) -> Result<Self, Self::Err> {
        if s.starts_with('(') && s.ends_with(')') {
            s = &s[1 .. s.len() - 1];
        }
        let mut parts = s.split(',');
        let group = parts.next().ok_or(TagRangeParseError("not enough tag components, expected `group,element`"))?;
        let elem = parts.next().ok_or(TagRangeParseError("not enough tag components, expected `element`"))?;
        if group.len() != 4 {
            return Err(TagRangeParseError("tag component `group` has an invalid length, must be 4"));
        }
        if elem.len() != 4 {
            return Err(TagRangeParseError("tag component `element` has an invalid length, must be 4"));
        }

        match (&group.as_bytes()[2..], &elem.as_bytes()[2..]) {
            (b"xx", b"xx") => {
                return Err(TagRangeParseError("unsupported tag range"));
            },
            (b"xx", _) => {
                // Group100
                let group = u16::from_str_radix(&group[..2], 16)
                    .map_err(|_e| TagRangeParseError("Invalid component `group`"))? << 8;
                let elem = u16::from_str_radix(elem, 16)
                    .map_err(|_e| TagRangeParseError("Invalid component `element`"))?;
                Ok(TagRange::Group100(Tag(group, elem)))
            },
            (_, b"xx") => {
                // Element100
                let group = u16::from_str_radix(group, 16)
                    .map_err(|_e| TagRangeParseError("Invalid component `group`"))?;
                let elem = u16::from_str_radix(&elem[..2], 16)
                    .map_err(|_e| TagRangeParseError("Invalid component `element`"))? << 8;
                Ok(TagRange::Element100(Tag(group, elem)))
            },
            (_, _) => {
                // single element
                let group = u16::from_str_radix(group, 16)
                    .map_err(|_e| TagRangeParseError("Invalid component `group`"))?;
                let elem = u16::from_str_radix(elem, 16)
                    .map_err(|_e| TagRangeParseError("Invalid component `element`"))?;
                Ok(TagRange::Single(Tag(group, elem)))
            }
        }
    }
}

/** Type trait for a dictionary of DICOM attributes. Attribute dictionaries provide the
 * means to convert a tag to an alias and vice versa, as well as a form of retrieving
 * additional information about the attribute.
 *
 * The methods herein have no generic parameters, so as to enable being
 * used as a trait object.
 */
pub trait DataDictionary {
    /// The type of the dictionary entry.
    type Entry: DictionaryEntry;

    /// Fetch an entry by its usual alias (e.g. "PatientName" or "SOPInstanceUID").
    /// Aliases are usually case sensitive and not separated by spaces.
    fn by_name(&self, name: &str) -> Option<&Self::Entry>;

    /// Fetch an entry by its tag.
    fn by_tag(&self, tag: Tag) -> Option<&Self::Entry>;
}

/// The dictionary entry data type, representing a DICOM attribute.
pub trait DictionaryEntry {
    /// The attribute tag or tag range.
    fn tag(&self) -> TagRange;
    /// The alias of the attribute, with no spaces, usually in UpperCamelCase.
    fn alias(&self) -> &str;
    /// The _typical_ value representation of the attribute.
    /// In some edge cases, an element might not have this VR.
    fn vr(&self) -> VR;
}

/// A data type for a dictionary entry with full ownership.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct DictionaryEntryBuf {
    /// The attribute tag or tag range
    pub tag: TagRange,
    /// The alias of the attribute, with no spaces, usually in UpperCamelCase
    pub alias: String,
    /// The _typical_  value representation of the attribute, although more may be applicable
    pub vr: VR,
}

impl DictionaryEntry for DictionaryEntryBuf {
    fn tag(&self) -> TagRange {
        self.tag
    }
    fn alias(&self) -> &str {
        self.alias.as_str()
    }
    fn vr(&self) -> VR {
        self.vr
    }
}

/// A data type for a dictionary entry with a string slice for its alias.
#[derive(Debug, PartialEq, Eq, Hash, Copy, Clone)]
pub struct DictionaryEntryRef<'a> {
    /// The attribute tag
    pub tag: TagRange,
    /// The alias of the attribute, with no spaces, usually InCapitalizedCamelCase
    pub alias: &'a str,
    /// The _typical_  value representation of the attribute
    pub vr: VR,
}

impl<'a> DictionaryEntry for DictionaryEntryRef<'a> {
    fn tag(&self) -> TagRange {
        self.tag
    }
    fn alias(&self) -> &str {
        self.alias
    }
    fn vr(&self) -> VR {
        self.vr
    }
}

/// Utility data structure that resolves to a DICOM attribute tag
/// at a later time.
#[derive(Debug)]
pub struct TagByName<N: AsRef<str>, D: DataDictionary> {
    dict: D,
    name: N,
}

impl<N: AsRef<str>, D: DataDictionary> TagByName<N, D> {
    /// Create a tag resolver by name using the given dictionary.
    pub fn new(dictionary: D, name: N) -> TagByName<N, D> {
        TagByName {
            dict: dictionary,
            name: name,
        }
    }
}

impl<N: AsRef<str>, D: DataDictionary> From<TagByName<N, D>> for Option<Tag> {
    fn from(tag: TagByName<N, D>) -> Option<Tag> {
        tag.dict.by_name(tag.name.as_ref()).map(|e| e.tag().inner())
    }
}

#[cfg(test)]
mod tests {
    use crate::header::Tag;
    use super::TagRange;

    #[test]
    fn test_parse_tag_range() {
        let tag: TagRange = "(1234,5678)".parse().unwrap();
        assert_eq!(tag, TagRange::Single(Tag(0x1234, 0x5678)));

        let tag: TagRange = "1234,5678".parse().unwrap();
        assert_eq!(tag, TagRange::Single(Tag(0x1234, 0x5678)));

        let tag: TagRange = "12xx,5678".parse().unwrap();
        assert_eq!(tag, TagRange::Group100(Tag(0x1200, 0x5678)));

        let tag: TagRange = "1234,56xx".parse().unwrap();
        assert_eq!(tag, TagRange::Element100(Tag(0x1234, 0x5600)));
    }
}