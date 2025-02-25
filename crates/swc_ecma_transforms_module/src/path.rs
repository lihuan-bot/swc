use anyhow::Error;
use pathdiff::diff_paths;
use std::{
    borrow::Cow,
    env::current_dir,
    path::{Component, PathBuf},
    sync::Arc,
};
use swc_atoms::JsWord;
use swc_common::FileName;
use swc_ecma_loader::resolve::Resolve;

pub trait ImportResolver {
    /// Resolves `target` as a string usable by the modules pass.
    ///
    /// The returned string will be used as a module specifier.
    fn resolve_import(&self, base: &FileName, module_specifier: &str) -> Result<JsWord, Error>;
}

/// [ImportResolver] implementation which just uses orignal source.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoopImportResolver;

impl ImportResolver for NoopImportResolver {
    fn resolve_import(&self, _: &FileName, module_specifier: &str) -> Result<JsWord, Error> {
        Ok(module_specifier.into())
    }
}

/// [ImportResolver] implementation for node.js
///
/// Supports [FileName::Real] and [FileName::Anon] for `base`, [FileName::Real]
/// and [FileName::Custom] for `target`. ([FileName::Custom] is used for core
/// modules)
#[derive(Debug, Clone, Default)]
pub struct NodeImportResolver<R>
where
    R: Resolve,
{
    resolver: R,
}

impl<R> NodeImportResolver<R>
where
    R: Resolve,
{
    pub fn new(resolver: R) -> Self {
        Self { resolver }
    }
}

impl<R> ImportResolver for NodeImportResolver<R>
where
    R: Resolve,
{
    fn resolve_import(&self, base: &FileName, module_specifier: &str) -> Result<JsWord, Error> {
        fn to_specifier(target_path: &str, is_file: Option<bool>) -> JsWord {
            let mut p = PathBuf::from(target_path);
            if is_file.unwrap_or_else(|| p.is_file()) {
                if let Some(v) = p.extension() {
                    if v == "ts" || v == "tsx" || v == "js" || v == "jsx" {
                        p.set_extension("");
                    }
                }
            }
            p.display().to_string().into()
        }

        let target = self.resolver.resolve(base, module_specifier);
        let target = match target {
            Ok(v) => v,
            Err(_) => return Ok(module_specifier.into()),
        };

        let target = match target {
            FileName::Real(v) => v,
            FileName::Custom(s) => return Ok(to_specifier(&s, None)),
            _ => {
                unreachable!(
                    "Node path provider does not support using `{:?}` as a target file name",
                    target
                )
            }
        };
        let base = match base {
            FileName::Real(v) => Cow::Borrowed(v),
            FileName::Anon => {
                if cfg!(target_arch = "wasm32") {
                    panic!("Please specify `filename`")
                } else {
                    Cow::Owned(current_dir().expect("failed to get current directory"))
                }
            }
            _ => {
                unreachable!(
                    "Node path provider does not support using `{:?}` as a base file name",
                    base
                )
            }
        };

        let is_file = target.is_file();

        let rel_path = diff_paths(
            &target,
            match base.parent() {
                Some(v) => v,
                None => &base,
            },
        );

        let rel_path = match rel_path {
            Some(v) => v,
            None => return Ok(to_specifier(&target.display().to_string(), Some(is_file))),
        };

        {
            // Check for `node_modules`.

            for component in rel_path.components() {
                match component {
                    Component::Prefix(_) => {}
                    Component::RootDir => {}
                    Component::CurDir => {}
                    Component::ParentDir => {}
                    Component::Normal(c) => {
                        if c == "node_modules" {
                            return Ok(module_specifier.into());
                        }
                    }
                }
            }
        }

        debug_assert!(
            !rel_path.is_absolute(),
            "Resolved path should not be absolute (in swc repository) but found {}\nbase: \
             {}\ntarget: {}",
            rel_path.display(),
            base.display(),
            target.display(),
        );

        let s = rel_path.to_string_lossy();
        let s = if s.starts_with('.') || s.starts_with('/') {
            s
        } else {
            Cow::Owned(format!("./{}", s))
        };
        if cfg!(target_os = "windows") {
            Ok(to_specifier(&s.replace('\\', "/"), Some(is_file)))
        } else {
            Ok(to_specifier(&s, Some(is_file)))
        }
    }
}

macro_rules! impl_ref {
    ($P:ident, $T:ty) => {
        impl<$P> ImportResolver for $T
        where
            $P: ImportResolver,
        {
            fn resolve_import(&self, base: &FileName, target: &str) -> Result<JsWord, Error> {
                (**self).resolve_import(base, target)
            }
        }
    };
}

impl_ref!(P, &'_ P);
impl_ref!(P, Box<P>);
impl_ref!(P, Arc<P>);
