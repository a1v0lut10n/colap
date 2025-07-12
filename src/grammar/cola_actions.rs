/// This file is maintained by rustemo but can be modified manually.
/// All manual changes will be preserved except non-doc comments.
use rustemo::{ValLoc, Context as C};
use rustemo::Token as RustemoToken;
use super::cola::{TokenKind, Context};
pub type Input = str;
pub type Ctx<'i> = Context<'i, Input>;
#[allow(dead_code)]
pub type Token<'i> = RustemoToken<'i, Input, TokenKind>;
pub type ColaCodeStart = ValLoc<String>;
pub fn cola_code_start(_ctx: &Ctx, token: Token) -> ColaCodeStart {
    ColaCodeStart::new(token.value.into(), Some(_ctx.location()))
}
pub type ColaCodeEnd = ValLoc<String>;
pub fn cola_code_end(_ctx: &Ctx, token: Token) -> ColaCodeEnd {
    ColaCodeEnd::new(token.value.into(), Some(_ctx.location()))
}
pub type HeadingLine = ValLoc<String>;
pub fn heading_line(_ctx: &Ctx, token: Token) -> HeadingLine {
    HeadingLine::new(token.value.into(), Some(_ctx.location()))
}
pub type Identifier = ValLoc<String>;
pub fn identifier(_ctx: &Ctx, token: Token) -> Identifier {
    Identifier::new(token.value.into(), Some(_ctx.location()))
}
pub type Number = ValLoc<String>;
pub fn number(_ctx: &Ctx, token: Token) -> Number {
    Number::new(token.value.into(), Some(_ctx.location()))
}
pub type ParagraphLine = ValLoc<String>;
pub fn paragraph_line(_ctx: &Ctx, token: Token) -> ParagraphLine {
    ParagraphLine::new(token.value.into(), Some(_ctx.location()))
}
pub type QuotedStringDouble = ValLoc<String>;
pub fn quoted_string_double(_ctx: &Ctx, token: Token) -> QuotedStringDouble {
    QuotedStringDouble::new(token.value.into(), Some(_ctx.location()))
}
pub type QuotedStringSingle = ValLoc<String>;
pub fn quoted_string_single(_ctx: &Ctx, token: Token) -> QuotedStringSingle {
    QuotedStringSingle::new(token.value.into(), Some(_ctx.location()))
}
pub type RegularCodeLine = ValLoc<String>;
pub fn regular_code_line(_ctx: &Ctx, token: Token) -> RegularCodeLine {
    RegularCodeLine::new(token.value.into(), Some(_ctx.location()))
}
pub type RegularCodeStartNamed = ValLoc<String>;
pub fn regular_code_start_named(_ctx: &Ctx, token: Token) -> RegularCodeStartNamed {
    RegularCodeStartNamed::new(token.value.into(), Some(_ctx.location()))
}
pub type RegularCodeStartUnnamed = ValLoc<String>;
pub fn regular_code_start_unnamed(_ctx: &Ctx, token: Token) -> RegularCodeStartUnnamed {
    RegularCodeStartUnnamed::new(token.value.into(), Some(_ctx.location()))
}
pub type RegularCodeEnd = ValLoc<String>;
pub fn regular_code_end(_ctx: &Ctx, token: Token) -> RegularCodeEnd {
    RegularCodeEnd::new(token.value.into(), Some(_ctx.location()))
}
pub type Cola = MarkdownItem0;
pub fn cola_markdown_item0(_ctx: &Ctx, markdown_item0: MarkdownItem0) -> Cola {
    markdown_item0
}
pub type MarkdownItem1 = Vec<MarkdownItem>;
pub fn markdown_item1_c1(
    _ctx: &Ctx,
    mut markdown_item1: MarkdownItem1,
    markdown_item: MarkdownItem,
) -> MarkdownItem1 {
    markdown_item1.push(markdown_item);
    markdown_item1
}
pub fn markdown_item1_markdown_item(
    _ctx: &Ctx,
    markdown_item: MarkdownItem,
) -> MarkdownItem1 {
    vec![markdown_item]
}
pub type MarkdownItem0 = Option<MarkdownItem1>;
pub fn markdown_item0_markdown_item1(
    _ctx: &Ctx,
    markdown_item1: MarkdownItem1,
) -> MarkdownItem0 {
    Some(markdown_item1)
}
pub fn markdown_item0_empty(_ctx: &Ctx) -> MarkdownItem0 {
    None
}
#[derive(Debug, Clone)]
pub enum MarkdownItem {
    HeadingLine(HeadingLine),
    CodeBlock(CodeBlock),
    ParagraphLine(ParagraphLine),
}
pub fn markdown_item_heading_line(
    _ctx: &Ctx,
    heading_line: HeadingLine,
) -> MarkdownItem {
    MarkdownItem::HeadingLine(heading_line)
}
pub fn markdown_item_code_block(_ctx: &Ctx, code_block: CodeBlock) -> MarkdownItem {
    MarkdownItem::CodeBlock(code_block)
}
pub fn markdown_item_paragraph_line(
    _ctx: &Ctx,
    paragraph_line: ParagraphLine,
) -> MarkdownItem {
    MarkdownItem::ParagraphLine(paragraph_line)
}
#[derive(Debug, Clone)]
pub enum CodeBlock {
    ColaCodeBlock(ColaCodeBlock),
    RegularCodeBlock(RegularCodeBlock),
}
pub fn code_block_cola_code_block(
    _ctx: &Ctx,
    cola_code_block: ColaCodeBlock,
) -> CodeBlock {
    CodeBlock::ColaCodeBlock(cola_code_block)
}
pub fn code_block_regular_code_block(
    _ctx: &Ctx,
    regular_code_block: RegularCodeBlock,
) -> CodeBlock {
    CodeBlock::RegularCodeBlock(regular_code_block)
}
#[derive(Debug, Clone)]
pub struct ColaCodeBlockBase {
    pub cola_code_start: ColaCodeStart,
    pub cola_syntax: ColaSyntax,
    pub cola_code_end: ColaCodeEnd,
}
pub type ColaCodeBlock = ValLoc<ColaCodeBlockBase>;
pub fn cola_code_block_c1(
    _ctx: &Ctx,
    cola_code_start: ColaCodeStart,
    cola_syntax: ColaSyntax,
    cola_code_end: ColaCodeEnd,
) -> ColaCodeBlock {
    ColaCodeBlock::new(
        ColaCodeBlockBase {
            cola_code_start,
            cola_syntax,
            cola_code_end,
        },
        Some(_ctx.location()),
    )
}
pub type ColaSyntax = Entity0;
pub fn cola_syntax_entity0(_ctx: &Ctx, entity0: Entity0) -> ColaSyntax {
    entity0
}
pub type Entity1 = Vec<Entity>;
pub fn entity1_c1(_ctx: &Ctx, mut entity1: Entity1, entity: Entity) -> Entity1 {
    entity1.push(entity);
    entity1
}
pub fn entity1_entity(_ctx: &Ctx, entity: Entity) -> Entity1 {
    vec![entity]
}
pub type Entity0 = Option<Entity1>;
pub fn entity0_entity1(_ctx: &Ctx, entity1: Entity1) -> Entity0 {
    Some(entity1)
}
pub fn entity0_empty(_ctx: &Ctx) -> Entity0 {
    None
}
#[derive(Debug, Clone)]
pub enum Entity {
    PluralEntity(PluralEntity),
    SingularEntity(SingularEntity),
}
pub fn entity_plural_entity(_ctx: &Ctx, plural_entity: PluralEntity) -> Entity {
    Entity::PluralEntity(plural_entity)
}
pub fn entity_singular_entity(_ctx: &Ctx, singular_entity: SingularEntity) -> Entity {
    Entity::SingularEntity(singular_entity)
}
#[derive(Debug, Clone)]
pub struct PluralEntityBase {
    pub identifier_1: Identifier,
    pub identifier_3: Identifier,
    pub entity_definition: EntityDefinition,
}
pub type PluralEntity = ValLoc<PluralEntityBase>;
pub fn plural_entity_c1(
    _ctx: &Ctx,
    identifier_1: Identifier,
    identifier_3: Identifier,
    entity_definition: EntityDefinition,
) -> PluralEntity {
    PluralEntity::new(
        PluralEntityBase {
            identifier_1,
            identifier_3,
            entity_definition,
        },
        Some(_ctx.location()),
    )
}
#[derive(Debug, Clone)]
pub struct SingularEntityBase {
    pub identifier: Identifier,
    pub entity_definition: EntityDefinition,
}
pub type SingularEntity = ValLoc<SingularEntityBase>;
pub fn singular_entity_c1(
    _ctx: &Ctx,
    identifier: Identifier,
    entity_definition: EntityDefinition,
) -> SingularEntity {
    SingularEntity::new(
        SingularEntityBase {
            identifier,
            entity_definition,
        },
        Some(_ctx.location()),
    )
}
pub type EntityDefinition = NestedBlock0;
pub fn entity_definition_nested_block0(
    _ctx: &Ctx,
    nested_block0: NestedBlock0,
) -> EntityDefinition {
    nested_block0
}
pub type NestedBlock1 = Vec<NestedBlock>;
pub fn nested_block1_c1(
    _ctx: &Ctx,
    mut nested_block1: NestedBlock1,
    nested_block: NestedBlock,
) -> NestedBlock1 {
    nested_block1.push(nested_block);
    nested_block1
}
pub fn nested_block1_nested_block(
    _ctx: &Ctx,
    nested_block: NestedBlock,
) -> NestedBlock1 {
    vec![nested_block]
}
pub type NestedBlock0 = Option<NestedBlock1>;
pub fn nested_block0_nested_block1(
    _ctx: &Ctx,
    nested_block1: NestedBlock1,
) -> NestedBlock0 {
    Some(nested_block1)
}
pub fn nested_block0_empty(_ctx: &Ctx) -> NestedBlock0 {
    None
}
#[derive(Debug, Clone)]
pub enum NestedBlock {
    FieldList(FieldList),
    Entity(Box<Entity>),
}
pub fn nested_block_field_list(_ctx: &Ctx, field_list: FieldList) -> NestedBlock {
    NestedBlock::FieldList(field_list)
}
pub fn nested_block_entity(_ctx: &Ctx, entity: Entity) -> NestedBlock {
    NestedBlock::Entity(Box::new(entity))
}
#[derive(Debug, Clone)]
pub struct FieldListC2Base {
    pub field_list: Box<FieldList>,
    pub field: Field,
}
pub type FieldListC2 = ValLoc<FieldListC2Base>;
#[derive(Debug, Clone)]
pub enum FieldList {
    Field(Field),
    C2(FieldListC2),
}
pub fn field_list_field(_ctx: &Ctx, field: Field) -> FieldList {
    FieldList::Field(field)
}
pub fn field_list_c2(_ctx: &Ctx, field_list: FieldList, field: Field) -> FieldList {
    FieldList::C2(
        FieldListC2::new(
            FieldListC2Base {
                field_list: Box::new(field_list),
                field,
            },
            Some(_ctx.location()),
        ),
    )
}
#[derive(Debug, Clone)]
pub struct FieldBase {
    pub identifier: Identifier,
    pub field_value: FieldValue,
}
pub type Field = ValLoc<FieldBase>;
pub fn field_c1(_ctx: &Ctx, identifier: Identifier, field_value: FieldValue) -> Field {
    Field::new(
        FieldBase {
            identifier,
            field_value,
        },
        Some(_ctx.location()),
    )
}
#[derive(Debug, Clone)]
pub enum FieldValue {
    QuotedStringDouble(QuotedStringDouble),
    QuotedStringSingle(QuotedStringSingle),
    Number(Number),
    BooleanTrue,
    BooleanFalse,
}
pub fn field_value_quoted_string_double(
    _ctx: &Ctx,
    quoted_string_double: QuotedStringDouble,
) -> FieldValue {
    FieldValue::QuotedStringDouble(quoted_string_double)
}
pub fn field_value_quoted_string_single(
    _ctx: &Ctx,
    quoted_string_single: QuotedStringSingle,
) -> FieldValue {
    FieldValue::QuotedStringSingle(quoted_string_single)
}
pub fn field_value_number(_ctx: &Ctx, number: Number) -> FieldValue {
    FieldValue::Number(number)
}
pub fn field_value_boolean_true(_ctx: &Ctx) -> FieldValue {
    FieldValue::BooleanTrue
}
pub fn field_value_boolean_false(_ctx: &Ctx) -> FieldValue {
    FieldValue::BooleanFalse
}
#[derive(Debug, Clone)]
pub struct RegularCodeBlockBase {
    pub regular_code_start: RegularCodeStart,
    pub regular_code_line0: RegularCodeLine0,
    pub regular_code_end: RegularCodeEnd,
}
pub type RegularCodeBlock = ValLoc<RegularCodeBlockBase>;
pub fn regular_code_block_c1(
    _ctx: &Ctx,
    regular_code_start: RegularCodeStart,
    regular_code_line0: RegularCodeLine0,
    regular_code_end: RegularCodeEnd,
) -> RegularCodeBlock {
    RegularCodeBlock::new(
        RegularCodeBlockBase {
            regular_code_start,
            regular_code_line0,
            regular_code_end,
        },
        Some(_ctx.location()),
    )
}
pub type RegularCodeLine1 = Vec<RegularCodeLine>;
pub fn regular_code_line1_c1(
    _ctx: &Ctx,
    mut regular_code_line1: RegularCodeLine1,
    regular_code_line: RegularCodeLine,
) -> RegularCodeLine1 {
    regular_code_line1.push(regular_code_line);
    regular_code_line1
}
pub fn regular_code_line1_regular_code_line(
    _ctx: &Ctx,
    regular_code_line: RegularCodeLine,
) -> RegularCodeLine1 {
    vec![regular_code_line]
}
pub type RegularCodeLine0 = Option<RegularCodeLine1>;
pub fn regular_code_line0_regular_code_line1(
    _ctx: &Ctx,
    regular_code_line1: RegularCodeLine1,
) -> RegularCodeLine0 {
    Some(regular_code_line1)
}
pub fn regular_code_line0_empty(_ctx: &Ctx) -> RegularCodeLine0 {
    None
}
#[derive(Debug, Clone)]
pub enum RegularCodeStart {
    RegularCodeStartNamed(RegularCodeStartNamed),
    RegularCodeStartUnnamed(RegularCodeStartUnnamed),
}
pub fn regular_code_start_regular_code_start_named(
    _ctx: &Ctx,
    regular_code_start_named: RegularCodeStartNamed,
) -> RegularCodeStart {
    RegularCodeStart::RegularCodeStartNamed(regular_code_start_named)
}
pub fn regular_code_start_regular_code_start_unnamed(
    _ctx: &Ctx,
    regular_code_start_unnamed: RegularCodeStartUnnamed,
) -> RegularCodeStart {
    RegularCodeStart::RegularCodeStartUnnamed(regular_code_start_unnamed)
}
