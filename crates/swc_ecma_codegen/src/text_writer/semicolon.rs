use super::{Result, WriteJs};
use swc_common::Span;

pub fn omit_trailing_semi<W: WriteJs>(w: W) -> impl WriteJs {
    OmitTrailingSemi {
        inner: w,
        pending_semi: false,
    }
}

#[derive(Debug, Clone)]
struct OmitTrailingSemi<W: WriteJs> {
    inner: W,
    pending_semi: bool,
}

macro_rules! with_semi {
    (
        $fn_name:ident
        (
            $(
                $arg_name:ident
                :
                $arg_ty:ty
            ),*
        )
    ) => {
        fn $fn_name(&mut self, $($arg_name: $arg_ty),* ) -> Result {
            self.commit_pending_semi()?;

            self.inner.$fn_name( $($arg_name),* )
        }
    };
}

impl<W: WriteJs> WriteJs for OmitTrailingSemi<W> {
    with_semi!(increase_indent());
    with_semi!(decrease_indent());

    fn write_semi(&mut self, _: Option<Span>) -> Result {
        self.pending_semi = true;
        Ok(())
    }

    with_semi!(write_space());
    with_semi!(write_comment(span: Span, s: &str));
    with_semi!(write_keyword(span: Option<Span>, s: &'static str));
    with_semi!(write_operator(span: Option<Span>, s: &str));
    with_semi!(write_param(s: &str));
    with_semi!(write_property(s: &str));
    with_semi!(write_line());
    with_semi!(write_lit(span: Span, s: &str));
    with_semi!(write_str_lit(span: Span, s: &str));
    with_semi!(write_str(s: &str));
    with_semi!(write_symbol(span: Span, s: &str));

    fn write_punct(&mut self, span: Option<Span>, s: &'static str) -> Result {
        match s {
            "\"" | "'" | "[" | "!" | "/" | "{" | "(" | "~" | "-" | "+" | "#" => {
                self.commit_pending_semi()?;
            }

            _ => {
                self.pending_semi = false;
            }
        }

        self.inner.write_punct(span, s)
    }

    fn target(&self) -> swc_ecma_ast::EsVersion {
        self.inner.target()
    }

    #[inline]
    fn care_about_srcmap(&self) -> bool {
        self.inner.care_about_srcmap()
    }
}

impl<W: WriteJs> OmitTrailingSemi<W> {
    fn commit_pending_semi(&mut self) -> Result {
        if self.pending_semi {
            self.inner.write_punct(None, ";")?;
            self.pending_semi = false;
        }
        Ok(())
    }
}
