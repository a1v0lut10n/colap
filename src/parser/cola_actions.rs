use super::cola::{Context, TokenKind};
use rustemo::Token as RustemoToken;
/// This file is maintained by rustemo but can be modified manually.
/// All manual changes will be preserved except non-doc comments.
use rustemo::{Context as C, ValLoc};
pub type Input = str;
pub type Ctx<'i> = Context<'i, Input>;
#[allow(dead_code)]
pub type Token<'i> = RustemoToken<'i, Input, TokenKind>;
pub type Identifier = ValLoc<String>;
pub fn identifier(_ctx: &Ctx, token: Token) -> Identifier {
    Identifier::new(token.value.into(), Some(_ctx.location()))
}
pub type Number = ValLoc<String>;
pub fn number(_ctx: &Ctx, token: Token) -> Number {
    Number::new(token.value.into(), Some(_ctx.location()))
}
pub type Reference = ValLoc<String>;
pub fn reference(_ctx: &Ctx, token: Token) -> Reference {
    Reference::new(token.value.into(), Some(_ctx.location()))
}
pub type QuotedStringDouble = ValLoc<String>;
pub fn quoted_string_double(_ctx: &Ctx, token: Token) -> QuotedStringDouble {
    QuotedStringDouble::new(token.value.into(), Some(_ctx.location()))
}
pub type QuotedStringSingle = ValLoc<String>;
pub fn quoted_string_single(_ctx: &Ctx, token: Token) -> QuotedStringSingle {
    QuotedStringSingle::new(token.value.into(), Some(_ctx.location()))
}
pub type BlockString = ValLoc<String>;
pub fn block_string(_ctx: &Ctx, token: Token) -> BlockString {
    BlockString::new(token.value.into(), Some(_ctx.location()))
}
pub type Cola = Entity0;
pub fn cola_entity0(_ctx: &Ctx, entity0: Entity0) -> Cola {
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
pub type EntityDefinition = Item0;
pub fn entity_definition_item0(_ctx: &Ctx, item0: Item0) -> EntityDefinition {
    item0
}
pub type Item1 = Vec<Item>;
pub fn item1_c1(_ctx: &Ctx, mut item1: Item1, item: Item) -> Item1 {
    item1.push(item);
    item1
}
pub fn item1_item(_ctx: &Ctx, item: Item) -> Item1 {
    vec![item]
}
pub type Item0 = Option<Item1>;
pub fn item0_item1(_ctx: &Ctx, item1: Item1) -> Item0 {
    Some(item1)
}
pub fn item0_empty(_ctx: &Ctx) -> Item0 {
    None
}
#[derive(Debug, Clone)]
pub enum Item {
    Field1(Field),
    Field2(Field),
    Entity(Box<Entity>),
}
pub fn item_field1(_ctx: &Ctx, field: Field) -> Item {
    Item::Field1(field)
}
pub fn item_field2(_ctx: &Ctx, field: Field) -> Item {
    Item::Field2(field)
}
pub fn item_entity(_ctx: &Ctx, entity: Entity) -> Item {
    Item::Entity(Box::new(entity))
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
    Scalar(Scalar),
    List(List),
}
pub fn field_value_scalar(_ctx: &Ctx, scalar: Scalar) -> FieldValue {
    FieldValue::Scalar(scalar)
}
pub fn field_value_list(_ctx: &Ctx, list: List) -> FieldValue {
    FieldValue::List(list)
}
#[derive(Debug, Clone)]
pub enum Scalar {
    QuotedStringDouble(QuotedStringDouble),
    QuotedStringSingle(QuotedStringSingle),
    BlockString(BlockString),
    Number(Number),
    BooleanTrue,
    BooleanFalse,
    Reference(Reference),
    Identifier(Identifier),
}
pub fn scalar_quoted_string_double(_ctx: &Ctx, quoted_string_double: QuotedStringDouble) -> Scalar {
    Scalar::QuotedStringDouble(quoted_string_double)
}
pub fn scalar_quoted_string_single(_ctx: &Ctx, quoted_string_single: QuotedStringSingle) -> Scalar {
    Scalar::QuotedStringSingle(quoted_string_single)
}
pub fn scalar_block_string(_ctx: &Ctx, block_string: BlockString) -> Scalar {
    Scalar::BlockString(block_string)
}
pub fn scalar_number(_ctx: &Ctx, number: Number) -> Scalar {
    Scalar::Number(number)
}
pub fn scalar_boolean_true(_ctx: &Ctx) -> Scalar {
    Scalar::BooleanTrue
}
pub fn scalar_boolean_false(_ctx: &Ctx) -> Scalar {
    Scalar::BooleanFalse
}
pub fn scalar_reference(_ctx: &Ctx, reference: Reference) -> Scalar {
    Scalar::Reference(reference)
}
pub fn scalar_identifier(_ctx: &Ctx, identifier: Identifier) -> Scalar {
    Scalar::Identifier(identifier)
}
#[derive(Debug, Clone)]
pub enum List {
    Scalar0(Scalar0),
    Scalar1(Scalar1),
}
pub fn list_scalar0(_ctx: &Ctx, scalar0: Scalar0) -> List {
    List::Scalar0(scalar0)
}
pub fn list_scalar1(_ctx: &Ctx, scalar1: Scalar1) -> List {
    List::Scalar1(scalar1)
}
pub type Scalar1 = Vec<Scalar>;
pub fn scalar1_c1(_ctx: &Ctx, mut scalar1: Scalar1, scalar: Scalar) -> Scalar1 {
    scalar1.push(scalar);
    scalar1
}
pub fn scalar1_scalar(_ctx: &Ctx, scalar: Scalar) -> Scalar1 {
    vec![scalar]
}
pub type Scalar0 = Option<Scalar1>;
pub fn scalar0_scalar1(_ctx: &Ctx, scalar1: Scalar1) -> Scalar0 {
    Some(scalar1)
}
pub fn scalar0_empty(_ctx: &Ctx) -> Scalar0 {
    None
}
