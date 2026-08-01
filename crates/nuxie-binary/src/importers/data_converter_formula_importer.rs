use super::*;

pub(super) fn imports_successfully(
    _object: &RuntimeObject,
    definition: &'static Definition,
    context: &ImportContext,
) -> Option<bool> {
    definition
        .is_a("FormulaToken")
        .then(|| context.latest(ImportStackKey::DataConverterFormula))
}

pub(super) fn update_context(definition: &'static Definition, context: &mut ImportContext) {
    if definition.name == "DataConverterFormula" {
        context.make_latest(ImportStackKey::DataConverterFormula);
    }
}
impl RuntimeFile {
    pub(crate) fn cpp_data_converter_formula_tokens(
        &self,
        data_converter_index: usize,
    ) -> Vec<&RuntimeObject> {
        self.cpp_data_converter_formula_output_tokens(data_converter_index)
            .into_iter()
            .map(|token| token.object)
            .collect()
    }

    pub(crate) fn cpp_data_converter_formula_output_tokens(
        &self,
        data_converter_index: usize,
    ) -> Vec<RuntimeFormulaOutputToken<'_>> {
        Self::cpp_data_converter_formula_output_queue(
            self.cpp_data_converter_formula_authored_tokens(data_converter_index),
        )
    }

    pub(crate) fn cpp_data_converter_formula_authored_tokens(
        &self,
        data_converter_index: usize,
    ) -> Vec<&RuntimeObject> {
        let Some(formula) = self.data_converter(data_converter_index) else {
            return Vec::new();
        };
        if formula.type_name != "DataConverterFormula" {
            return Vec::new();
        }

        let mut latest_formula_index = None;
        let mut current_converter_index = 0usize;
        let mut tokens = Vec::new();

        for (file_index, object) in self.objects.iter().enumerate() {
            if self.import_status(file_index) != Some(RuntimeImportStatus::Imported) {
                continue;
            }

            let Some(object) = object.as_ref() else {
                continue;
            };
            let Some(definition) = definition_by_type_key(object.type_key) else {
                continue;
            };

            if definition.is_a("DataConverter") {
                if definition.name == "DataConverterFormula" {
                    latest_formula_index = Some(current_converter_index);
                }
                current_converter_index += 1;
                continue;
            }

            if definition.is_a("FormulaToken") && latest_formula_index == Some(data_converter_index)
            {
                tokens.push(object);
            }
        }

        tokens
    }

    pub(crate) fn cpp_data_converter_formula_output_queue<'a>(
        tokens: Vec<&'a RuntimeObject>,
    ) -> Vec<RuntimeFormulaOutputToken<'a>> {
        let mut operations_stack: Vec<&'a RuntimeObject> = Vec::new();
        let mut output_queue: Vec<RuntimeFormulaOutputToken<'a>> = Vec::new();
        let mut arguments_count = BTreeMap::new();

        for (token_index, token) in tokens.iter().enumerate() {
            match token.type_name {
                "FormulaTokenValue" | "FormulaTokenInput" => {
                    output_queue.push(RuntimeFormulaOutputToken::new(token, &arguments_count))
                }
                "FormulaTokenOperation" => {
                    while operations_stack.last().is_some_and(|operation| {
                        operation.type_name != "FormulaTokenParenthesisOpen"
                            && Self::cpp_formula_token_precedence(operation)
                                >= Self::cpp_formula_token_precedence(token)
                    }) {
                        let operation = operations_stack.pop().expect("stack has last token");
                        output_queue
                            .push(RuntimeFormulaOutputToken::new(operation, &arguments_count));
                    }
                    operations_stack.push(*token);
                }
                "FormulaTokenParenthesisOpen" | "FormulaTokenFunction" => {
                    let argument_count = if tokens
                        .get(token_index + 1)
                        .is_some_and(|next| next.type_name == "FormulaTokenParenthesisClose")
                    {
                        0
                    } else {
                        1
                    };
                    arguments_count.insert(token.id, argument_count);
                    operations_stack.push(*token);
                }
                "FormulaTokenParenthesisClose" => {
                    while operations_stack.last().is_some_and(|operation| {
                        operation.type_name != "FormulaTokenParenthesisOpen"
                            && operation.type_name != "FormulaTokenFunction"
                    }) {
                        let operation = operations_stack.pop().expect("stack has last token");
                        output_queue
                            .push(RuntimeFormulaOutputToken::new(operation, &arguments_count));
                    }
                    if let Some(opening_token) = operations_stack.pop()
                        && opening_token.type_name == "FormulaTokenFunction"
                    {
                        output_queue.push(RuntimeFormulaOutputToken::new(
                            opening_token,
                            &arguments_count,
                        ));
                    }
                }
                "FormulaTokenArgumentSeparator" if !operations_stack.is_empty() => {
                    if let Some(argument_token) = operations_stack
                        .iter()
                        .rev()
                        .find(|operation| arguments_count.contains_key(&operation.id))
                    {
                        let count = arguments_count
                            .get(&argument_token.id)
                            .copied()
                            .unwrap_or(0);
                        arguments_count.insert(argument_token.id, count + 1);
                    }
                    while operations_stack.last().is_some_and(|operation| {
                        operation.type_name != "FormulaTokenParenthesisOpen"
                            && operation.type_name != "FormulaTokenFunction"
                    }) {
                        let operation = operations_stack.pop().expect("stack has last token");
                        output_queue
                            .push(RuntimeFormulaOutputToken::new(operation, &arguments_count));
                    }
                }
                _ => {}
            }
        }

        while let Some(operation) = operations_stack.pop() {
            if operation.type_name != "FormulaTokenParenthesisOpen" {
                output_queue.push(RuntimeFormulaOutputToken::new(operation, &arguments_count));
            }
        }

        output_queue
    }

    pub(crate) fn cpp_formula_token_precedence(token: &RuntimeObject) -> u8 {
        let Some(definition) = definition_by_type_key(token.type_key) else {
            return 0;
        };
        if definition.is_a("FormulaTokenParenthesis") {
            return 1;
        }
        if definition.name == "FormulaTokenOperation" {
            return match token.uint_property("operationType").unwrap_or(0) {
                0 | 1 => 2,
                2 | 3 => 3,
                _ => 0,
            };
        }
        0
    }
}
