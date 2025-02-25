use crate::{Ident, Str, Tokens};
use is_macro::Is;
use string_enum::StringEnum;
use swc_atoms::JsWord;
use swc_common::{ast_node, EqIgnoreSpan, Span};

#[ast_node("SelectorList")]
pub struct SelectorList {
    pub span: Span,
    pub children: Vec<ComplexSelector>,
}

#[ast_node("ComplexSelector")]
pub struct ComplexSelector {
    pub span: Span,
    pub children: Vec<ComplexSelectorChildren>,
}

#[ast_node]
#[derive(Is)]
pub enum ComplexSelectorChildren {
    #[tag("CompoundSelector")]
    CompoundSelector(CompoundSelector),
    #[tag("Combinator")]
    Combinator(Combinator),
}

/// e.g. `foo.c1.c2`
#[ast_node("CompoundSelector")]
pub struct CompoundSelector {
    pub span: Span,
    /// "&"
    pub nesting_selector: Option<NestingSelector>,
    pub type_selector: Option<TypeSelector>,
    pub subclass_selectors: Vec<SubclassSelector>,
}

#[ast_node("Combinator")]
pub struct Combinator {
    pub span: Span,
    pub value: CombinatorValue,
}

#[derive(StringEnum, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, EqIgnoreSpan)]
pub enum CombinatorValue {
    /// ` `
    Descendant,

    /// `+`
    NextSibling,

    /// `>`
    Child,

    /// `~`
    LaterSibling,

    /// `||`
    Column,
}

#[ast_node("NestingSelector")]
pub struct NestingSelector {
    pub span: Span,
}

#[ast_node]
pub enum TypeSelector {
    #[tag("TagNameSelector")]
    TagName(TagNameSelector),
    #[tag("UniversalSelector")]
    Universal(UniversalSelector),
}

#[ast_node("TagNameSelector")]
pub struct TagNameSelector {
    pub span: Span,
    pub name: WqName,
}

#[ast_node("UniversalSelector")]
pub struct UniversalSelector {
    pub span: Span,
    pub prefix: Option<NsPrefix>,
}

#[ast_node("NsPrefix")]
pub struct NsPrefix {
    pub span: Span,
    pub prefix: Option<Ident>,
}

#[ast_node("WqName")]
pub struct WqName {
    pub span: Span,
    pub prefix: Option<NsPrefix>,
    pub value: Ident,
}

#[ast_node]
pub enum SubclassSelector {
    #[tag("IdSelector")]
    Id(IdSelector),

    #[tag("ClassSelector")]
    Class(ClassSelector),

    #[tag("AttributeSelector")]
    Attribute(AttributeSelector),

    #[tag("PseudoClassSelector")]
    PseudoClass(PseudoClassSelector),

    #[tag("PseudoElementSelector")]
    PseudoElement(PseudoElementSelector),

    #[tag("AtSelector")]
    At(AtSelector),
}

#[ast_node("IdSelector")]
pub struct IdSelector {
    pub span: Span,
    /// Does not include `#`
    pub text: Ident,
}

#[ast_node("ClassSelector")]
pub struct ClassSelector {
    pub span: Span,
    /// Does not include `.`
    pub text: Ident,
}

#[ast_node("AttributeSelector")]
pub struct AttributeSelector {
    pub span: Span,
    pub name: WqName,
    pub matcher: Option<AttributeSelectorMatcher>,
    pub value: Option<AttributeSelectorValue>,
    pub modifier: Option<AttributeSelectorModifier>,
}

#[derive(StringEnum, Clone, Copy, Eq, PartialEq, PartialOrd, Ord, Hash, EqIgnoreSpan)]
pub enum AttributeSelectorMatcherValue {
    /// `=`
    Equals,

    /// `~=`
    Tilde,

    /// `|=`
    Bar,

    /// `^=`
    Caret,

    /// `$=`
    Dollar,

    /// `*=`
    Asterisk,
}

#[ast_node("AttributeSelectorMatcher")]
pub struct AttributeSelectorMatcher {
    pub span: Span,
    pub value: AttributeSelectorMatcherValue,
}

#[ast_node]
pub enum AttributeSelectorValue {
    #[tag("String")]
    Str(Str),

    #[tag("Ident")]
    Ident(Ident),
}

#[ast_node("AttributeSelectorModifier")]
pub struct AttributeSelectorModifier {
    pub span: Span,
    pub value: Ident,
}

#[ast_node("PseudoClassSelector")]
pub struct PseudoClassSelector {
    pub span: Span,
    pub name: Ident,
    pub children: Option<PseudoSelectorChildren>,
}

#[ast_node("PseudoElementSelector")]
pub struct PseudoElementSelector {
    pub span: Span,
    pub name: Ident,
    pub children: Option<Tokens>,
}

/// Type for `@top-center`. Allowwed in only some contexts.
#[ast_node("AtSelector")]
pub struct AtSelector {
    pub span: Span,
    pub text: Ident,
}

#[ast_node]
pub enum PseudoSelectorChildren {
    #[tag("Nth")]
    Nth(Nth),

    #[tag("Tokens")]
    Tokens(Tokens),
}

#[ast_node("Nth")]
pub struct Nth {
    pub span: Span,
    pub nth: NthValue,
    pub selector_list: Option<SelectorList>,
}

#[ast_node("AnPlusB")]
pub struct AnPlusB {
    pub span: Span,
    pub a: Option<i32>,
    pub a_raw: Option<JsWord>,
    pub b: Option<i32>,
    pub b_raw: Option<JsWord>,
}

#[ast_node]
pub enum NthValue {
    #[tag("AnPlusB")]
    AnPlusB(AnPlusB),

    #[tag("Ident")]
    Ident(Ident),
}
