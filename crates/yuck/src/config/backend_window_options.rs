use std::collections::HashMap;

use anyhow::Result;
use simplexpr::{dynval::DynVal, eval::EvalError, SimplExpr};

use crate::{enum_parse, error::DiagResult, value::coords};
use eww_shared_util::VarName;

use super::{attributes::Attributes, window_definition::EnumParseError};

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    EnumParseError(#[from] EnumParseError),
    #[error(transparent)]
    CoordsError(#[from] coords::Error),
    #[error(transparent)]
    EvalError(#[from] EvalError),
}

/// Backend-specific options of a window
/// Unevaluated form of [`BackendWindowOptions`]
#[derive(Clone, Debug, Eq, PartialEq, serde::Serialize)]
pub struct BackendWindowOptionsDef {
    pub wayland: WlBackendWindowOptionsDef,
}

impl BackendWindowOptionsDef {
    pub fn eval(&self, local_variables: &HashMap<VarName, DynVal>) -> Result<BackendWindowOptions, Error> {
        Ok(BackendWindowOptions { wayland: self.wayland.eval(local_variables)? })
    }

    pub fn from_attrs(attrs: &mut Attributes) -> DiagResult<Self> {
        //let struts = attrs.ast_optional("reserve")?;
        //let window_type = attrs.ast_optional("windowtype")?;
        let wayland = WlBackendWindowOptionsDef {
            exclusive: attrs.ast_optional("exclusive")?,
            focusable: attrs.ast_optional("focusable")?,
            namespace: attrs.ast_optional("namespace")?,
        };

        Ok(Self { wayland })
    }
}

/// Backend-specific options of a window that are backend
#[derive(Debug, Clone, serde::Serialize, PartialEq)]
pub struct BackendWindowOptions {
    pub wayland: WlBackendWindowOptions,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct WlBackendWindowOptions {
    pub exclusive: bool,
    pub focusable: bool,
    pub namespace: Option<String>,
}

/// Unevaluated form of [`WlBackendWindowOptions`]
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize)]
pub struct WlBackendWindowOptionsDef {
    pub exclusive: Option<SimplExpr>,
    pub focusable: Option<SimplExpr>,
    pub namespace: Option<SimplExpr>,
}

impl WlBackendWindowOptionsDef {
    fn eval(&self, local_variables: &HashMap<VarName, DynVal>) -> Result<WlBackendWindowOptions, EvalError> {
        Ok(WlBackendWindowOptions {
            exclusive: eval_opt_expr_as_bool(&self.exclusive, false, local_variables)?,
            focusable: eval_opt_expr_as_bool(&self.focusable, false, local_variables)?,
            namespace: match &self.namespace {
                Some(expr) => Some(expr.eval(local_variables)?.as_string()?),
                None => None,
            },
        })
    }
}

fn eval_opt_expr_as_bool(
    opt_expr: &Option<SimplExpr>,
    default: bool,
    local_variables: &HashMap<VarName, DynVal>,
) -> Result<bool, EvalError> {
    Ok(match opt_expr {
        Some(expr) => expr.eval(local_variables)?.as_bool()?,
        None => default,
    })
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, smart_default::SmartDefault, serde::Serialize)]
pub enum Side {
    #[default]
    Top,
    Left,
    Right,
    Bottom,
}

impl std::str::FromStr for Side {
    type Err = EnumParseError;

    fn from_str(s: &str) -> Result<Side, Self::Err> {
        enum_parse! { "side", s,
            "l" | "left" => Side::Left,
            "r" | "right" => Side::Right,
            "t" | "top" => Side::Top,
            "b" | "bottom" => Side::Bottom,
        }
    }
}
