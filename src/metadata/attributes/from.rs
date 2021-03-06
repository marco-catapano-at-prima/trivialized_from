use crate::metadata::attributes::PathDetection;
use crate::metadata::{as_name, ParseError};
use std::collections::HashSet;
use syn::spanned::Spanned;
use syn::{Attribute, Meta, NestedMeta};

#[derive(Default)]
pub(crate) struct FromMetadata {
    path_detections: HashSet<PathDetection>,
}

//TODO: refactor, see TransformMetadata
impl FromMetadata {
    pub(crate) fn maybe_from(attr: &Attribute) -> Result<Option<Self>, Vec<ParseError>> {
        if as_name(&attr.path).ne("From") {
            return Ok(None);
        }

        let meta = attr.parse_meta().map_err(|e| {
            vec![ParseError {
                message: format!("Unable to successfully parse this attribute because \"{}\". Expected format is: #[From(Type1,...,TypeN)]", e),
                span: attr.span().clone(),
            }]
        })?;

        let meta_list = match meta {
            Meta::List(list) => Ok(list),
            Meta::Path(path) => Err(vec![ParseError {
                message: format!("#[From] attribute does not support Path format. Expected format is: #[From(Type1,...,TypeN)]"),
                span: path.span().clone()
            }]),
            Meta::NameValue(name_value) => Err(vec![ParseError {
                message: format!("#[From] attribute does not support NameValue format. Expected format is: #[From(Type1,...,TypeN)]"),
                span: name_value.span().clone()
            }])
        }?;

        let extracted_types: Vec<_> = meta_list.nested.iter().map(|nested_meta| {
            let meta = match nested_meta {
                NestedMeta::Meta(meta) => Ok(meta),
                NestedMeta::Lit(lit) => Err(ParseError {
                    message: format!("Literal NestedMeta detected in #[From] MetaList. Expected format is: #[From(Type1,...,TypeN)]"),
                    span: lit.span().clone()
                })
            }?;

            match meta {
                Meta::Path(ref path) => Ok(PathDetection {
                    stringified: as_name(&path),
                    span: path.span().clone()
                }),
                _ => Err(ParseError {
                    message: format!("NestedMeta Path is needed in #[From] MetaList. Expected format is: #[From(Type1,...,TypeN)]"),
                    span: meta.span().clone()
                })
            }
        }).collect();

        let (types, errors): (Vec<_>, Vec<_>) =
            extracted_types.into_iter().partition(Result::is_ok);

        if errors.len() > 0 {
            return Err(errors.into_iter().filter_map(Result::err).collect());
        }

        Ok(Some(FromMetadata {
            path_detections: types.into_iter().filter_map(Result::ok).collect(),
        }))
    }

    pub(crate) fn types(&self) -> Result<Vec<syn::Type>, Vec<ParseError>> {
        let result_types: Vec<_> = self
            .path_detections
            .iter()
            .map(|detection| {
                syn::parse_str::<syn::Type>(&detection.stringified).map_err(|e| ParseError {
                    message: format!("Unable to parse type from this token: {}", e),
                    span: detection.span.clone(),
                })
            })
            .collect();

        let (types, errors): (Vec<_>, Vec<_>) = result_types.into_iter().partition(Result::is_ok);

        if errors.len() > 0 {
            return Err(errors.into_iter().filter_map(Result::err).collect());
        }

        Ok(types.into_iter().filter_map(Result::ok).collect())
    }

    pub(crate) fn merge(self, other: Self) -> Self {
        Self {
            path_detections: self
                .path_detections
                .into_iter()
                .chain(other.path_detections)
                .collect(),
        }
    }
}
