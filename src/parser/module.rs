//! Module parsing functionality

use super::NixFile;
use crate::Result;
use std::collections::HashMap;
use std::path::PathBuf;

/// A parser for `NixOS` modules
pub struct ModuleParser;

/// A parsed `NixOS` module
#[derive(Debug, Clone)]
pub struct ParsedModule {
    /// The underlying parsed file
    pub file: NixFile,
    /// Module imports
    pub imports: Vec<PathBuf>,
    /// Module options
    pub options: HashMap<String, OptionDefinition>,
    /// Module configuration
    pub config: HashMap<String, serde_json::Value>,
    /// Module metadata
    pub meta: ModuleMeta,
}

/// Definition of a module option
#[derive(Debug, Clone)]
pub struct OptionDefinition {
    /// Option type
    pub option_type: String,
    /// Default value if any
    pub default: Option<serde_json::Value>,
    /// Option description
    pub description: Option<String>,
    /// Example value
    pub example: Option<serde_json::Value>,
}

/// Module metadata
#[derive(Debug, Clone, Default)]
pub struct ModuleMeta {
    /// Module maintainers
    pub maintainers: Vec<String>,
    /// Module description
    pub description: Option<String>,
}

impl ModuleParser {
    /// Parse a `NixFile` as a module
    pub fn parse(file: &NixFile) -> Result<ParsedModule> {
        // Convert to our AST representation
        let ast = file.to_ast()?;
        
        // Extract module components
        let mut imports = Vec::new();
        let mut options = HashMap::new();
        let mut config = HashMap::new();
        let mut meta = ModuleMeta::default();
        
        // A module can be either:
        // 1. A function that takes arguments and returns an attribute set
        // 2. An attribute set directly
        match &ast {
            super::ast::NixAst::Function { body, .. } => {
                // Module is a function, extract from body
                Self::extract_from_ast(body, &mut imports, &mut options, &mut config, &mut meta)?;
            }
            super::ast::NixAst::AttrSet { bindings, .. } => {
                // Module is directly an attribute set
                Self::extract_from_bindings(bindings, &mut imports, &mut options, &mut config, &mut meta)?;
            }
            _ => {
                return Err(crate::NixDomainError::ParseError(
                    "Module must be either a function or an attribute set".to_string()
                ));
            }
        }
        
        Ok(ParsedModule {
            file: file.clone(),
            imports,
            options,
            config,
            meta,
        })
    }
    
    /// Extract module components from an AST node
    fn extract_from_ast(
        ast: &super::ast::NixAst,
        imports: &mut Vec<PathBuf>,
        options: &mut HashMap<String, OptionDefinition>,
        config: &mut HashMap<String, serde_json::Value>,
        meta: &mut ModuleMeta,
    ) -> Result<()> {
        match ast {
            super::ast::NixAst::AttrSet { bindings, .. } => {
                Self::extract_from_bindings(bindings, imports, options, config, meta)?;
            }
            _ => {
                return Err(crate::NixDomainError::ParseError(
                    "Module body must be an attribute set".to_string()
                ));
            }
        }
        Ok(())
    }
    
    /// Extract module components from attribute set bindings
    fn extract_from_bindings(
        bindings: &[super::ast::Binding],
        imports: &mut Vec<PathBuf>,
        options: &mut HashMap<String, OptionDefinition>,
        config: &mut HashMap<String, serde_json::Value>,
        meta: &mut ModuleMeta,
    ) -> Result<()> {
        use super::ast::{AttrPathSegment, BindingValue};
        
        for binding in bindings {
            if let Some(AttrPathSegment::Identifier(key)) = binding.attr_path.segments.first() {
                match key.as_str() {
                    "imports" => {
                        if let BindingValue::Value(value) = &binding.value {
                            Self::extract_imports(value, imports)?;
                        }
                    }
                    "options" => {
                        if let BindingValue::Value(value) = &binding.value {
                            Self::extract_options(value, options)?;
                        }
                    }
                    "config" => {
                        if let BindingValue::Value(value) = &binding.value {
                            Self::extract_config(value, config)?;
                        }
                    }
                    "meta" => {
                        if let BindingValue::Value(value) = &binding.value {
                            Self::extract_meta(value, meta)?;
                        }
                    }
                    _ => {
                        // Other top-level attributes are treated as config
                        if let BindingValue::Value(value) = &binding.value {
                            let json_value = Self::ast_to_json(value)?;
                            config.insert(key.clone(), json_value);
                        }
                    }
                }
            }
        }
        
        Ok(())
    }
    
    /// Extract imports from an AST node
    fn extract_imports(ast: &super::ast::NixAst, imports: &mut Vec<PathBuf>) -> Result<()> {
        match ast {
            super::ast::NixAst::List(items) => {
                for item in items {
                    match item {
                        super::ast::NixAst::String(s) => {
                            imports.push(PathBuf::from(s));
                        }
                        super::ast::NixAst::Path(p) => {
                            imports.push(p.clone());
                        }
                        _ => {
                            // Skip non-path imports for now
                        }
                    }
                }
            }
            _ => {
                return Err(crate::NixDomainError::ParseError(
                    "imports must be a list".to_string()
                ));
            }
        }
        Ok(())
    }
    
    /// Extract options from an AST node
    fn extract_options(
        ast: &super::ast::NixAst,
        options: &mut HashMap<String, OptionDefinition>,
    ) -> Result<()> {
        match ast {
            super::ast::NixAst::AttrSet { bindings, .. } => {
                use super::ast::{AttrPathSegment, BindingValue};
                
                for binding in bindings {
                    let option_path = binding.attr_path.segments.iter()
                        .filter_map(|seg| {
                            if let AttrPathSegment::Identifier(name) = seg {
                                Some(name.as_str())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(".");
                    
                    if let BindingValue::Value(value) = &binding.value {
                        let option_def = Self::extract_option_definition(value)?;
                        options.insert(option_path, option_def);
                    }
                }
            }
            _ => {
                return Err(crate::NixDomainError::ParseError(
                    "options must be an attribute set".to_string()
                ));
            }
        }
        Ok(())
    }
    
    /// Extract option definition from an AST node
    fn extract_option_definition(ast: &super::ast::NixAst) -> Result<OptionDefinition> {
        let mut option_type = String::from("any");
        let mut default = None;
        let mut description = None;
        let mut example = None;
        
        if let super::ast::NixAst::AttrSet { bindings, .. } = ast {
            use super::ast::{AttrPathSegment, BindingValue};
            
            for binding in bindings {
                if let Some(AttrPathSegment::Identifier(key)) = binding.attr_path.segments.first() {
                    if let BindingValue::Value(value) = &binding.value {
                        match key.as_str() {
                            "type" => {
                                option_type = Self::extract_type_name(value)?;
                            }
                            "default" => {
                                default = Some(Self::ast_to_json(value)?);
                            }
                            "description" => {
                                if let super::ast::NixAst::String(s) = value {
                                    description = Some(s.clone());
                                }
                            }
                            "example" => {
                                example = Some(Self::ast_to_json(value)?);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        
        Ok(OptionDefinition {
            option_type,
            default,
            description,
            example,
        })
    }
    
    /// Extract type name from a type expression
    fn extract_type_name(ast: &super::ast::NixAst) -> Result<String> {
        match ast {
            super::ast::NixAst::Identifier(name) => Ok(name.clone()),
            super::ast::NixAst::Select { expr, attr_path, .. } => {
                // Handle types like `types.str` or `lib.types.int`
                let path_parts: Vec<String> = attr_path.segments.iter()
                    .filter_map(|seg| {
                        if let super::ast::AttrPathSegment::Identifier(name) = seg {
                            Some(name.clone())
                        } else {
                            None
                        }
                    })
                    .collect();
                
                if let Some(last) = path_parts.last() {
                    Ok(last.clone())
                } else {
                    Ok("any".to_string())
                }
            }
            _ => Ok("any".to_string()),
        }
    }
    
    /// Extract config from an AST node
    fn extract_config(
        ast: &super::ast::NixAst,
        config: &mut HashMap<String, serde_json::Value>,
    ) -> Result<()> {
        match ast {
            super::ast::NixAst::AttrSet { bindings, .. } => {
                use super::ast::{AttrPathSegment, BindingValue};
                
                for binding in bindings {
                    let config_path = binding.attr_path.segments.iter()
                        .filter_map(|seg| {
                            if let AttrPathSegment::Identifier(name) = seg {
                                Some(name.as_str())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join(".");
                    
                    if let BindingValue::Value(value) = &binding.value {
                        let json_value = Self::ast_to_json(value)?;
                        config.insert(config_path, json_value);
                    }
                }
            }
            _ => {
                return Err(crate::NixDomainError::ParseError(
                    "config must be an attribute set".to_string()
                ));
            }
        }
        Ok(())
    }
    
    /// Extract meta information from an AST node
    fn extract_meta(ast: &super::ast::NixAst, meta: &mut ModuleMeta) -> Result<()> {
        if let super::ast::NixAst::AttrSet { bindings, .. } = ast {
            use super::ast::{AttrPathSegment, BindingValue};
            
            for binding in bindings {
                if let Some(AttrPathSegment::Identifier(key)) = binding.attr_path.segments.first() {
                    if let BindingValue::Value(value) = &binding.value {
                        match key.as_str() {
                            "description" => {
                                if let super::ast::NixAst::String(s) = value {
                                    meta.description = Some(s.clone());
                                }
                            }
                            "maintainers" => {
                                if let super::ast::NixAst::List(items) = value {
                                    meta.maintainers = items.iter()
                                        .filter_map(|item| {
                                            if let super::ast::NixAst::String(s) = item {
                                                Some(s.clone())
                                            } else {
                                                None
                                            }
                                        })
                                        .collect();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }
    
    /// Convert an AST node to JSON
    fn ast_to_json(ast: &super::ast::NixAst) -> Result<serde_json::Value> {
        use super::ast::NixAst;
        
        Ok(match ast {
            NixAst::Integer(i) => serde_json::json!(i),
            NixAst::Float(f) => serde_json::json!(f),
            NixAst::String(s) => serde_json::json!(s),
            NixAst::Bool(b) => serde_json::json!(b),
            NixAst::Null => serde_json::Value::Null,
            NixAst::Path(p) => serde_json::json!(p.to_string_lossy()),
            NixAst::List(items) => {
                let json_items: Result<Vec<_>> = items.iter()
                    .map(|item| Self::ast_to_json(item))
                    .collect();
                serde_json::json!(json_items?)
            }
            NixAst::AttrSet { bindings, .. } => {
                use super::ast::{AttrPathSegment, BindingValue};
                
                let mut map = serde_json::Map::new();
                for binding in bindings {
                    if let Some(AttrPathSegment::Identifier(key)) = binding.attr_path.segments.first() {
                        if let BindingValue::Value(value) = &binding.value {
                            let json_value = Self::ast_to_json(value)?;
                            map.insert(key.clone(), json_value);
                        }
                    }
                }
                serde_json::Value::Object(map)
            }
            // For complex expressions, convert to string representation
            _ => serde_json::json!(format!("{:?}", ast)),
        })
    }
} 