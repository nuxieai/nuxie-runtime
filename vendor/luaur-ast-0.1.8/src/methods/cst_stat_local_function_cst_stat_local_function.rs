use crate::records::cst_node::CstNode;
use crate::records::cst_stat_local_function::CstStatLocalFunction;
use crate::records::ast_array::AstArray;
use crate::records::cst_attr_list::CstAttrList;
use crate::records::position::Position;
use crate::rtti::CstNodeClass;

impl CstStatLocalFunction {
    pub fn new(local_keyword_position: Position, function_keyword_position: Position) -> Self {
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            attr_lists: AstArray::default(),
            local_keyword_position,
            function_keyword_position,
        }
    }

    pub fn new_with_attr_lists(
        attr_lists: AstArray<*mut CstAttrList>,
        local_keyword_position: Position,
        function_keyword_position: Position,
    ) -> Self {
        luaur_common::LUAU_ASSERT!(luaur_common::FFlag::LuauCstAttr.get());
        Self {
            base: CstNode {
                class_index: <Self as CstNodeClass>::CLASS_INDEX,
            },
            attr_lists,
            local_keyword_position,
            function_keyword_position,
        }
    }
}
