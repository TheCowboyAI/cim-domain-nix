// Copyright 2025 Cowboy AI, LLC.

//! Nix Value Objects - The 9 fundamental Nix types
//!
//! This module implements type-safe Rust representations of Nix's value types.
//!
//! ## Type System
//!
//! Nix has 9 types:
//!
//! ### Primitive Types (7)
//! - `NixString` - Text values with string context
//! - `NixInteger` - 64-bit signed integers
//! - `NixFloat` - 64-bit floating point numbers
//! - `NixBool` - Boolean true/false
//! - `NixNull` - The null value
//! - `NixPath` - Filesystem paths
//! - `NixLookupPath` - Nix search path entries like `<nixpkgs>`
//!
//! ### Compound Types (2)
//! - `NixAttrset` - Attribute sets (key-value mappings)
//! - `NixList` - Ordered lists of values
//!
//! ## Usage
//!
//! ```rust
//! use cim_domain_nix::nix::value_objects::*;
//!
//! // Create Nix values
//! let s = NixString::new("hello");
//! let i = NixInteger::new(42);
//! let b = NixBool::new(true);
//!
//! // Work with attribute sets
//! let mut attrs = NixAttrset::new();
//! attrs.insert("name".to_string(), NixValue::String(s));
//! attrs.insert("count".to_string(), NixValue::Integer(i));
//! ```

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when working with Nix values
#[derive(Debug, Error, Clone, PartialEq)]
pub enum NixValueError {
    /// Invalid path
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Invalid lookup path
    #[error("Invalid lookup path: {0}")]
    InvalidLookupPath(String),

    /// Type mismatch
    #[error("Type mismatch: expected {expected}, got {got}")]
    TypeMismatch { expected: String, got: String },

    /// Missing attribute
    #[error("Missing attribute: {0}")]
    MissingAttribute(String),

    /// Invalid attribute key
    #[error("Invalid attribute key: {0}")]
    InvalidAttributeKey(String),
}

/// Result type for Nix value operations
pub type Result<T> = std::result::Result<T, NixValueError>;

// ============================================================================
// Primitive Types (7)
// ============================================================================

/// Nix String - Text with optional string context
///
/// String context tracks derivation dependencies and is used for
/// proper garbage collection in Nix.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixString {
    /// The string value
    pub value: String,
    /// Optional string context (derivation paths, etc.)
    pub context: Vec<String>,
}

impl NixString {
    /// Create a new string without context
    pub fn new(value: impl Into<String>) -> Self {
        Self {
            value: value.into(),
            context: vec![],
        }
    }

    /// Create a string with context
    pub fn with_context(value: impl Into<String>, context: Vec<String>) -> Self {
        Self {
            value: value.into(),
            context,
        }
    }

    /// Check if string has context
    pub fn has_context(&self) -> bool {
        !self.context.is_empty()
    }
}

impl fmt::Display for NixString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "\"{}\"", self.value)
    }
}

/// Nix Integer - 64-bit signed integer
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct NixInteger {
    /// The integer value
    pub value: i64,
}

impl NixInteger {
    /// Create a new integer
    pub fn new(value: i64) -> Self {
        Self { value }
    }
}

impl fmt::Display for NixInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Nix Float - 64-bit floating point number
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct NixFloat {
    /// The float value
    pub value: f64,
}

impl NixFloat {
    /// Create a new float
    pub fn new(value: f64) -> Self {
        Self { value }
    }
}

impl fmt::Display for NixFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Nix Bool - Boolean true/false
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixBool {
    /// The boolean value
    pub value: bool,
}

impl NixBool {
    /// Create a new boolean
    pub fn new(value: bool) -> Self {
        Self { value }
    }

    /// The true value
    pub fn true_value() -> Self {
        Self { value: true }
    }

    /// The false value
    pub fn false_value() -> Self {
        Self { value: false }
    }
}

impl fmt::Display for NixBool {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.value)
    }
}

/// Nix Null - The null value
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixNull;

impl NixNull {
    /// Create the null value
    pub fn new() -> Self {
        Self
    }
}

impl Default for NixNull {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NixNull {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "null")
    }
}

/// Nix Path - Filesystem path
///
/// Paths in Nix are first-class values used for imports and file references.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixPath {
    /// The path value
    pub path: PathBuf,
}

impl NixPath {
    /// Create a new path
    pub fn new(path: impl Into<PathBuf>) -> Result<Self> {
        let path = path.into();
        if path.as_os_str().is_empty() {
            return Err(NixValueError::InvalidPath("Path cannot be empty".into()));
        }
        Ok(Self { path })
    }

    /// Get the path as a string
    pub fn as_str(&self) -> &str {
        self.path.to_str().unwrap_or("")
    }

    /// Check if path is absolute
    pub fn is_absolute(&self) -> bool {
        self.path.is_absolute()
    }

    /// Check if path is relative
    pub fn is_relative(&self) -> bool {
        !self.is_absolute()
    }
}

impl fmt::Display for NixPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.path.display())
    }
}

/// Nix Lookup Path - Nix search path entry like `<nixpkgs>`
///
/// Lookup paths reference entries in NIX_PATH environment variable.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NixLookupPath {
    /// The lookup path name (e.g., "nixpkgs")
    pub name: String,
}

impl NixLookupPath {
    /// Create a new lookup path
    pub fn new(name: impl Into<String>) -> Result<Self> {
        let name = name.into();
        if name.is_empty() {
            return Err(NixValueError::InvalidLookupPath(
                "Lookup path name cannot be empty".into(),
            ));
        }
        Ok(Self { name })
    }
}

impl fmt::Display for NixLookupPath {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "<{}>", self.name)
    }
}

// ============================================================================
// Compound Types (2)
// ============================================================================

/// Nix List - Ordered collection of values
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NixList {
    /// The list elements
    pub elements: Vec<NixValue>,
}

impl NixList {
    /// Create an empty list
    pub fn new() -> Self {
        Self {
            elements: Vec::new(),
        }
    }

    /// Create a list from elements
    pub fn from_vec(elements: Vec<NixValue>) -> Self {
        Self { elements }
    }

    /// Get the length of the list
    pub fn len(&self) -> usize {
        self.elements.len()
    }

    /// Check if the list is empty
    pub fn is_empty(&self) -> bool {
        self.elements.is_empty()
    }

    /// Push a value to the list
    pub fn push(&mut self, value: NixValue) {
        self.elements.push(value);
    }

    /// Get an element by index
    pub fn get(&self, index: usize) -> Option<&NixValue> {
        self.elements.get(index)
    }
}

impl Default for NixList {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NixList {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[")?;
        for (i, elem) in self.elements.iter().enumerate() {
            if i > 0 {
                write!(f, " ")?;
            }
            write!(f, "{}", elem)?;
        }
        write!(f, "]")
    }
}

/// Nix Attribute Set - Key-value mapping
///
/// This is the fundamental data structure in Nix, similar to objects/dicts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NixAttrset {
    /// The attributes (key-value pairs)
    pub attributes: HashMap<String, NixValue>,
    /// Whether this is a recursive attrset (rec { })
    pub recursive: bool,
}

impl NixAttrset {
    /// Create an empty attribute set
    pub fn new() -> Self {
        Self {
            attributes: HashMap::new(),
            recursive: false,
        }
    }

    /// Create a recursive attribute set
    pub fn new_recursive() -> Self {
        Self {
            attributes: HashMap::new(),
            recursive: true,
        }
    }

    /// Insert an attribute
    pub fn insert(&mut self, key: String, value: NixValue) {
        self.attributes.insert(key, value);
    }

    /// Get an attribute
    pub fn get(&self, key: &str) -> Option<&NixValue> {
        self.attributes.get(key)
    }

    /// Check if attribute exists
    pub fn contains(&self, key: &str) -> bool {
        self.attributes.contains_key(key)
    }

    /// Get number of attributes
    pub fn len(&self) -> usize {
        self.attributes.len()
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.attributes.is_empty()
    }

    /// Get all keys
    pub fn keys(&self) -> impl Iterator<Item = &String> {
        self.attributes.keys()
    }

    /// Get all values
    pub fn values(&self) -> impl Iterator<Item = &NixValue> {
        self.attributes.values()
    }
}

impl Default for NixAttrset {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for NixAttrset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.recursive {
            write!(f, "rec {{ ")?;
        } else {
            write!(f, "{{ ")?;
        }
        for (i, (key, value)) in self.attributes.iter().enumerate() {
            if i > 0 {
                write!(f, "; ")?;
            }
            write!(f, "{} = {}", key, value)?;
        }
        write!(f, " }}")
    }
}

// ============================================================================
// The NixValue Enum - Union of all Nix types
// ============================================================================

/// NixValue - Union type representing any Nix value
///
/// This is the main type for working with Nix values in a type-safe way.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NixValue {
    /// String value
    String(NixString),
    /// Integer value
    Integer(NixInteger),
    /// Float value
    Float(NixFloat),
    /// Boolean value
    Bool(NixBool),
    /// Null value
    Null(NixNull),
    /// Path value
    Path(NixPath),
    /// Lookup path value
    LookupPath(NixLookupPath),
    /// List value
    List(NixList),
    /// Attribute set value
    Attrset(NixAttrset),
}

impl NixValue {
    /// Get the type name as a string
    pub fn type_name(&self) -> &'static str {
        match self {
            NixValue::String(_) => "string",
            NixValue::Integer(_) => "integer",
            NixValue::Float(_) => "float",
            NixValue::Bool(_) => "bool",
            NixValue::Null(_) => "null",
            NixValue::Path(_) => "path",
            NixValue::LookupPath(_) => "lookup_path",
            NixValue::List(_) => "list",
            NixValue::Attrset(_) => "attrset",
        }
    }

    /// Try to get as string
    pub fn as_string(&self) -> Result<&NixString> {
        match self {
            NixValue::String(s) => Ok(s),
            _ => Err(NixValueError::TypeMismatch {
                expected: "string".into(),
                got: self.type_name().into(),
            }),
        }
    }

    /// Try to get as integer
    pub fn as_integer(&self) -> Result<i64> {
        match self {
            NixValue::Integer(i) => Ok(i.value),
            _ => Err(NixValueError::TypeMismatch {
                expected: "integer".into(),
                got: self.type_name().into(),
            }),
        }
    }

    /// Try to get as float
    pub fn as_float(&self) -> Result<f64> {
        match self {
            NixValue::Float(f) => Ok(f.value),
            _ => Err(NixValueError::TypeMismatch {
                expected: "float".into(),
                got: self.type_name().into(),
            }),
        }
    }

    /// Try to get as boolean
    pub fn as_bool(&self) -> Result<bool> {
        match self {
            NixValue::Bool(b) => Ok(b.value),
            _ => Err(NixValueError::TypeMismatch {
                expected: "bool".into(),
                got: self.type_name().into(),
            }),
        }
    }

    /// Try to get as attribute set
    pub fn as_attrset(&self) -> Result<&NixAttrset> {
        match self {
            NixValue::Attrset(a) => Ok(a),
            _ => Err(NixValueError::TypeMismatch {
                expected: "attrset".into(),
                got: self.type_name().into(),
            }),
        }
    }

    /// Try to get as list
    pub fn as_list(&self) -> Result<&NixList> {
        match self {
            NixValue::List(l) => Ok(l),
            _ => Err(NixValueError::TypeMismatch {
                expected: "list".into(),
                got: self.type_name().into(),
            }),
        }
    }

    /// Check if value is null
    pub fn is_null(&self) -> bool {
        matches!(self, NixValue::Null(_))
    }
}

impl fmt::Display for NixValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NixValue::String(s) => write!(f, "{}", s),
            NixValue::Integer(i) => write!(f, "{}", i),
            NixValue::Float(fl) => write!(f, "{}", fl),
            NixValue::Bool(b) => write!(f, "{}", b),
            NixValue::Null(n) => write!(f, "{}", n),
            NixValue::Path(p) => write!(f, "{}", p),
            NixValue::LookupPath(lp) => write!(f, "{}", lp),
            NixValue::List(l) => write!(f, "{}", l),
            NixValue::Attrset(a) => write!(f, "{}", a),
        }
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nix_string_creation() {
        let s = NixString::new("hello");
        assert_eq!(s.value, "hello");
        assert!(!s.has_context());
    }

    #[test]
    fn test_nix_string_with_context() {
        let s = NixString::with_context("hello", vec!["/nix/store/abc".to_string()]);
        assert_eq!(s.value, "hello");
        assert!(s.has_context());
        assert_eq!(s.context.len(), 1);
    }

    #[test]
    fn test_nix_integer() {
        let i = NixInteger::new(42);
        assert_eq!(i.value, 42);
        assert_eq!(format!("{}", i), "42");
    }

    #[test]
    fn test_nix_float() {
        let f = NixFloat::new(3.14);
        assert_eq!(f.value, 3.14);
    }

    #[test]
    fn test_nix_bool() {
        let t = NixBool::true_value();
        let f = NixBool::false_value();
        assert!(t.value);
        assert!(!f.value);
    }

    #[test]
    fn test_nix_null() {
        let n = NixNull::new();
        assert_eq!(format!("{}", n), "null");
    }

    #[test]
    fn test_nix_path() {
        let p = NixPath::new("/nix/store/abc").unwrap();
        assert!(p.is_absolute());
        assert!(!p.is_relative());
    }

    #[test]
    fn test_nix_path_empty_fails() {
        let result = NixPath::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_nix_lookup_path() {
        let lp = NixLookupPath::new("nixpkgs").unwrap();
        assert_eq!(lp.name, "nixpkgs");
        assert_eq!(format!("{}", lp), "<nixpkgs>");
    }

    #[test]
    fn test_nix_lookup_path_empty_fails() {
        let result = NixLookupPath::new("");
        assert!(result.is_err());
    }

    #[test]
    fn test_nix_list() {
        let mut list = NixList::new();
        assert!(list.is_empty());
        assert_eq!(list.len(), 0);

        list.push(NixValue::Integer(NixInteger::new(1)));
        list.push(NixValue::Integer(NixInteger::new(2)));
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_nix_attrset() {
        let mut attrs = NixAttrset::new();
        assert!(attrs.is_empty());
        assert_eq!(attrs.len(), 0);

        attrs.insert("name".to_string(), NixValue::String(NixString::new("test")));
        attrs.insert("count".to_string(), NixValue::Integer(NixInteger::new(42)));

        assert_eq!(attrs.len(), 2);
        assert!(attrs.contains("name"));
        assert!(attrs.contains("count"));
    }

    #[test]
    fn test_nix_attrset_recursive() {
        let attrs = NixAttrset::new_recursive();
        assert!(attrs.recursive);
    }

    #[test]
    fn test_nix_value_type_name() {
        let s = NixValue::String(NixString::new("hello"));
        assert_eq!(s.type_name(), "string");

        let i = NixValue::Integer(NixInteger::new(42));
        assert_eq!(i.type_name(), "integer");

        let n = NixValue::Null(NixNull::new());
        assert_eq!(n.type_name(), "null");
    }

    #[test]
    fn test_nix_value_as_string() {
        let s = NixValue::String(NixString::new("hello"));
        let result = s.as_string();
        assert!(result.is_ok());
        assert_eq!(result.unwrap().value, "hello");
    }

    #[test]
    fn test_nix_value_as_string_type_mismatch() {
        let i = NixValue::Integer(NixInteger::new(42));
        let result = i.as_string();
        assert!(result.is_err());
    }

    #[test]
    fn test_nix_value_as_integer() {
        let i = NixValue::Integer(NixInteger::new(42));
        let result = i.as_integer();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), 42);
    }

    #[test]
    fn test_nix_value_is_null() {
        let n = NixValue::Null(NixNull::new());
        assert!(n.is_null());

        let i = NixValue::Integer(NixInteger::new(42));
        assert!(!i.is_null());
    }

    #[test]
    fn test_nix_value_display() {
        let s = NixValue::String(NixString::new("hello"));
        assert_eq!(format!("{}", s), "\"hello\"");

        let i = NixValue::Integer(NixInteger::new(42));
        assert_eq!(format!("{}", i), "42");

        let b = NixValue::Bool(NixBool::true_value());
        assert_eq!(format!("{}", b), "true");
    }

    // ============================================================================
    // Additional Tests for 90% Coverage
    // ============================================================================

    #[test]
    fn test_nix_string_display() {
        let s = NixString::new("world");
        assert_eq!(format!("{}", s), "\"world\"");
    }

    #[test]
    fn test_nix_float_display() {
        let f = NixFloat::new(2.718);
        let display = format!("{}", f);
        assert!(display.contains("2.718"));
    }

    #[test]
    fn test_nix_bool_new() {
        let b = NixBool::new(true);
        assert!(b.value);
        let b = NixBool::new(false);
        assert!(!b.value);
    }

    #[test]
    fn test_nix_bool_display() {
        let t = NixBool::true_value();
        assert_eq!(format!("{}", t), "true");
        let f = NixBool::false_value();
        assert_eq!(format!("{}", f), "false");
    }

    #[test]
    fn test_nix_null_default() {
        let n = NixNull::default();
        assert_eq!(format!("{}", n), "null");
    }

    #[test]
    fn test_nix_path_relative() {
        let p = NixPath::new("./relative/path").unwrap();
        assert!(p.is_relative());
        assert!(!p.is_absolute());
        assert_eq!(p.as_str(), "./relative/path");
    }

    #[test]
    fn test_nix_path_display() {
        let p = NixPath::new("/nix/store/test").unwrap();
        assert_eq!(format!("{}", p), "/nix/store/test");
    }

    #[test]
    fn test_nix_list_default() {
        let list = NixList::default();
        assert!(list.is_empty());
    }

    #[test]
    fn test_nix_list_from_vec() {
        let elements = vec![
            NixValue::Integer(NixInteger::new(1)),
            NixValue::Integer(NixInteger::new(2)),
        ];
        let list = NixList::from_vec(elements);
        assert_eq!(list.len(), 2);
    }

    #[test]
    fn test_nix_list_get() {
        let mut list = NixList::new();
        list.push(NixValue::Integer(NixInteger::new(42)));

        assert!(list.get(0).is_some());
        assert!(list.get(1).is_none());
    }

    #[test]
    fn test_nix_list_display() {
        let elements = vec![
            NixValue::Integer(NixInteger::new(1)),
            NixValue::Integer(NixInteger::new(2)),
        ];
        let list = NixList::from_vec(elements);
        assert_eq!(format!("{}", list), "[1 2]");
    }

    #[test]
    fn test_nix_list_display_empty() {
        let list = NixList::new();
        assert_eq!(format!("{}", list), "[]");
    }

    #[test]
    fn test_nix_attrset_default() {
        let attrs = NixAttrset::default();
        assert!(attrs.is_empty());
        assert!(!attrs.recursive);
    }

    #[test]
    fn test_nix_attrset_keys_and_values() {
        let mut attrs = NixAttrset::new();
        attrs.insert("a".to_string(), NixValue::Integer(NixInteger::new(1)));
        attrs.insert("b".to_string(), NixValue::Integer(NixInteger::new(2)));

        let keys: Vec<&String> = attrs.keys().collect();
        assert_eq!(keys.len(), 2);

        let values: Vec<&NixValue> = attrs.values().collect();
        assert_eq!(values.len(), 2);
    }

    #[test]
    fn test_nix_attrset_display() {
        let mut attrs = NixAttrset::new();
        attrs.insert("x".to_string(), NixValue::Integer(NixInteger::new(1)));
        let display = format!("{}", attrs);
        assert!(display.contains("x = 1"));
        assert!(display.contains("{"));
        assert!(display.contains("}"));
    }

    #[test]
    fn test_nix_attrset_recursive_display() {
        let mut attrs = NixAttrset::new_recursive();
        attrs.insert("x".to_string(), NixValue::Integer(NixInteger::new(1)));
        let display = format!("{}", attrs);
        assert!(display.contains("rec {"));
    }

    #[test]
    fn test_nix_value_as_float() {
        let f = NixValue::Float(NixFloat::new(3.14));
        let result = f.as_float();
        assert!(result.is_ok());
        assert!((result.unwrap() - 3.14).abs() < 0.001);
    }

    #[test]
    fn test_nix_value_as_float_type_mismatch() {
        let i = NixValue::Integer(NixInteger::new(42));
        let result = i.as_float();
        assert!(result.is_err());
    }

    #[test]
    fn test_nix_value_as_bool() {
        let b = NixValue::Bool(NixBool::true_value());
        let result = b.as_bool();
        assert!(result.is_ok());
        assert!(result.unwrap());
    }

    #[test]
    fn test_nix_value_as_bool_type_mismatch() {
        let s = NixValue::String(NixString::new("true"));
        let result = s.as_bool();
        assert!(result.is_err());
    }

    #[test]
    fn test_nix_value_as_attrset() {
        let attrs = NixAttrset::new();
        let a = NixValue::Attrset(attrs);
        let result = a.as_attrset();
        assert!(result.is_ok());
    }

    #[test]
    fn test_nix_value_as_attrset_type_mismatch() {
        let l = NixValue::List(NixList::new());
        let result = l.as_attrset();
        assert!(result.is_err());
    }

    #[test]
    fn test_nix_value_as_list() {
        let l = NixValue::List(NixList::new());
        let result = l.as_list();
        assert!(result.is_ok());
    }

    #[test]
    fn test_nix_value_as_list_type_mismatch() {
        let a = NixValue::Attrset(NixAttrset::new());
        let result = a.as_list();
        assert!(result.is_err());
    }

    #[test]
    fn test_nix_value_display_float() {
        let f = NixValue::Float(NixFloat::new(1.5));
        assert_eq!(format!("{}", f), "1.5");
    }

    #[test]
    fn test_nix_value_display_null() {
        let n = NixValue::Null(NixNull::new());
        assert_eq!(format!("{}", n), "null");
    }

    #[test]
    fn test_nix_value_display_path() {
        let p = NixValue::Path(NixPath::new("/test").unwrap());
        assert_eq!(format!("{}", p), "/test");
    }

    #[test]
    fn test_nix_value_display_lookup_path() {
        let lp = NixValue::LookupPath(NixLookupPath::new("nixpkgs").unwrap());
        assert_eq!(format!("{}", lp), "<nixpkgs>");
    }

    #[test]
    fn test_nix_value_display_list() {
        let l = NixValue::List(NixList::new());
        assert_eq!(format!("{}", l), "[]");
    }

    #[test]
    fn test_nix_value_display_attrset() {
        let a = NixValue::Attrset(NixAttrset::new());
        assert!(format!("{}", a).contains("{"));
    }

    #[test]
    fn test_nix_value_type_names() {
        assert_eq!(NixValue::Float(NixFloat::new(0.0)).type_name(), "float");
        assert_eq!(NixValue::Path(NixPath::new("/").unwrap()).type_name(), "path");
        assert_eq!(NixValue::LookupPath(NixLookupPath::new("x").unwrap()).type_name(), "lookup_path");
        assert_eq!(NixValue::List(NixList::new()).type_name(), "list");
        assert_eq!(NixValue::Attrset(NixAttrset::new()).type_name(), "attrset");
    }

    #[test]
    fn test_nix_value_error_display() {
        let err = NixValueError::InvalidPath("empty".into());
        assert_eq!(format!("{}", err), "Invalid path: empty");

        let err = NixValueError::InvalidLookupPath("bad".into());
        assert_eq!(format!("{}", err), "Invalid lookup path: bad");

        let err = NixValueError::TypeMismatch {
            expected: "string".into(),
            got: "integer".into(),
        };
        assert!(format!("{}", err).contains("expected string"));
        assert!(format!("{}", err).contains("got integer"));

        let err = NixValueError::MissingAttribute("name".into());
        assert_eq!(format!("{}", err), "Missing attribute: name");

        let err = NixValueError::InvalidAttributeKey("bad key".into());
        assert_eq!(format!("{}", err), "Invalid attribute key: bad key");
    }

    #[test]
    fn test_nix_integer_ordering() {
        let a = NixInteger::new(10);
        let b = NixInteger::new(20);
        assert!(a < b);
        assert!(b > a);
        assert_eq!(a, NixInteger::new(10));
    }
}
