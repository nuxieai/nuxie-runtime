use super::formula::{
    formula_token_argument_separator::FormulaTokenArgumentSeparator,
    formula_token_function::FormulaTokenFunction, formula_token_input::FormulaTokenInput,
    formula_token_operation::FormulaTokenOperation,
    formula_token_parenthesis::FormulaTokenParenthesis,
    formula_token_parenthesis_close::FormulaTokenParenthesisClose,
    formula_token_parenthesis_open::FormulaTokenParenthesisOpen,
    formula_token_value::FormulaTokenValue,
};
use crate::mechanical_port::source::{
    core::CoreHandle,
    data_bind::{
        data_context::RuntimeDataContextHandle,
        data_values::{
            data_type::DataType, data_value::DataValue, data_value_number::DataValueNumber,
            data_value_symbol_list_index::DataValueSymbolListIndex,
        },
    },
    generated::data_bind::converters::data_converter_formula_base::DataConverterFormulaBase,
    math::random::RandomProvider,
    viewmodel::viewmodel_instance_value::{ValueDependentHandle, ViewModelInstanceValue},
};
use std::collections::HashMap;
#[repr(u32)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RandomMode {
    Once = 0,
    Always = 1,
    SourceChange = 2,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FunctionType {
    Min,
    Max,
    Round,
    Ceil,
    Floor,
    Sqrt,
    Pow,
    Exp,
    Log,
    Cosine,
    Sine,
    Tangent,
    Acosine,
    Asine,
    Atangent,
    Atangent2,
    Random,
    Other,
}
#[derive(Clone, Debug)]
enum FormulaTokenKind {
    Value(f32),
    Input,
    Operation(u32),
    Function(FunctionType),
    Parenthesis,
    Open,
    Close,
    ArgumentSeparator,
    Other,
}
pub trait FormulaSource {
    fn add_dependent(&mut self, dependent: CoreHandle);
    fn remove_dependent(&mut self, dependent: &CoreHandle);
}
pub struct DataConverterFormula {
    pub base: DataConverterFormulaBase,
    tokens: Vec<CoreHandle>,
    output_queue: Vec<CoreHandle>,
    randoms: Vec<f32>,
    argument_counts: HashMap<(usize, usize, u64), i32>,
    is_instance: bool,
    source: Option<CoreHandle>,
    output: DataValueNumber,
}

impl Default for DataConverterFormula {
    fn default() -> Self {
        Self {
            base: DataConverterFormulaBase::default(),
            tokens: Vec::new(),
            output_queue: Vec::new(),
            randoms: Vec::new(),
            argument_counts: HashMap::new(),
            is_instance: false,
            source: None,
            output: DataValueNumber::default(),
        }
    }
}

impl DataConverterFormula {
    pub fn new(random_mode: RandomMode) -> Self {
        let mut formula = Self::default();
        formula.base.set_random_mode_value_value(random_mode as u32);
        formula
    }
    pub fn output_type(&self) -> DataType {
        DataType::Number
    }
    // Read the actual imported token on every evaluation: data bindings can
    // mutate its authored value between conversions.
    fn token_kind(token: &CoreHandle) -> FormulaTokenKind {
        token
            .with(|token| {
                if let Some(value) = token.as_any().downcast_ref::<FormulaTokenValue>() {
                    FormulaTokenKind::Value(value.base.operation_value())
                } else if token.as_any().is::<FormulaTokenInput>() {
                    FormulaTokenKind::Input
                } else if let Some(operation) =
                    token.as_any().downcast_ref::<FormulaTokenOperation>()
                {
                    FormulaTokenKind::Operation(operation.base.operation_type())
                } else if let Some(function) = token.as_any().downcast_ref::<FormulaTokenFunction>()
                {
                    FormulaTokenKind::Function(match function.base.function_type() {
                        0 => FunctionType::Min,
                        1 => FunctionType::Max,
                        2 => FunctionType::Round,
                        3 => FunctionType::Ceil,
                        4 => FunctionType::Floor,
                        5 => FunctionType::Sqrt,
                        6 => FunctionType::Pow,
                        7 => FunctionType::Exp,
                        8 => FunctionType::Log,
                        9 => FunctionType::Cosine,
                        10 => FunctionType::Sine,
                        11 => FunctionType::Tangent,
                        12 => FunctionType::Acosine,
                        13 => FunctionType::Asine,
                        14 => FunctionType::Atangent,
                        15 => FunctionType::Atangent2,
                        16 => FunctionType::Random,
                        _ => FunctionType::Other,
                    })
                } else if token.as_any().is::<FormulaTokenParenthesisOpen>() {
                    FormulaTokenKind::Open
                } else if token.as_any().is::<FormulaTokenParenthesisClose>() {
                    FormulaTokenKind::Close
                } else if token.as_any().is::<FormulaTokenParenthesis>() {
                    FormulaTokenKind::Parenthesis
                } else if token.as_any().is::<FormulaTokenArgumentSeparator>() {
                    FormulaTokenKind::ArgumentSeparator
                } else {
                    FormulaTokenKind::Other
                }
            })
            .expect("a formula retains its authored token")
    }
    fn precedence(token: &CoreHandle) -> i32 {
        match Self::token_kind(token) {
            FormulaTokenKind::Parenthesis
            | FormulaTokenKind::Open
            | FormulaTokenKind::Close
            | FormulaTokenKind::Function(_) => 1,
            FormulaTokenKind::Operation(0 | 1) => 2,
            FormulaTokenKind::Operation(2 | 3) => 3,
            _ => 0,
        }
    }
    pub fn calculate_formula(&mut self) {
        let mut operations: Vec<CoreHandle> = Vec::new();
        for (index, token) in self.tokens.iter().enumerate() {
            match Self::token_kind(token) {
                FormulaTokenKind::Value(_) | FormulaTokenKind::Input => {
                    self.output_queue.push(token.clone())
                }
                FormulaTokenKind::Operation(_) => {
                    while operations.last().is_some_and(|top| {
                        !matches!(Self::token_kind(top), FormulaTokenKind::Open)
                            && Self::precedence(top) >= Self::precedence(token)
                    }) {
                        self.output_queue.push(operations.pop().unwrap());
                    }
                    operations.push(token.clone());
                }
                FormulaTokenKind::Open | FormulaTokenKind::Function(_) => {
                    let next = self.tokens.get(index + 1);
                    self.argument_counts.insert(
                        token.identity_key(),
                        if next.is_some_and(|next| {
                            matches!(Self::token_kind(next), FormulaTokenKind::Close)
                        }) {
                            0
                        } else {
                            1
                        },
                    );
                    operations.push(token.clone());
                }
                FormulaTokenKind::Close => {
                    while operations.last().is_some_and(|top| {
                        !matches!(
                            Self::token_kind(top),
                            FormulaTokenKind::Open | FormulaTokenKind::Function(_)
                        )
                    }) {
                        self.output_queue.push(operations.pop().unwrap());
                    }
                    if let Some(open) = operations.pop() {
                        if matches!(Self::token_kind(&open), FormulaTokenKind::Function(_)) {
                            self.output_queue.push(open);
                        }
                    }
                }
                FormulaTokenKind::ArgumentSeparator if !operations.is_empty() => {
                    for candidate in operations.iter().rev() {
                        if let Some(count) = self.argument_counts.get_mut(&candidate.identity_key())
                        {
                            *count += 1;
                            break;
                        }
                    }
                    while operations.last().is_some_and(|top| {
                        !matches!(
                            Self::token_kind(top),
                            FormulaTokenKind::Open | FormulaTokenKind::Function(_)
                        )
                    }) {
                        self.output_queue.push(operations.pop().unwrap());
                    }
                }
                _ => {}
            }
        }
        while let Some(operation) = operations.pop() {
            if !matches!(Self::token_kind(&operation), FormulaTokenKind::Open) {
                self.output_queue.push(operation);
            }
        }
    }
    fn positive_mod(left: f32, right: f32) -> f32 {
        let range = right.abs();
        let mut value = left % range;
        if value < 0.0 {
            value += range;
        }
        value
    }
    fn apply_operation(left: f32, right: f32, operation: u32) -> f32 {
        match operation {
            0 => left + right,
            1 => left - right,
            2 => left * right,
            3 => left / right,
            4 => Self::positive_mod(left, right),
            _ => 0.0,
        }
    }
    fn get_random(&mut self, index: usize) -> f32 {
        if self.base.random_mode_value() == RandomMode::Always as u32 {
            return RandomProvider::generate_random_float();
        }
        while self.randoms.len() <= index {
            self.randoms.push(RandomProvider::generate_random_float());
        }
        self.randoms[index]
    }
    fn apply_function(&mut self, stack: &mut Vec<f32>, function: FunctionType, total: i32) -> f32 {
        let mut arguments = Vec::new();
        for _ in 0..total {
            if let Some(value) = stack.pop() {
                arguments.push(value);
            }
        }
        match function {
            FunctionType::Min => arguments
                .iter()
                .copied()
                .reduce(|a, b| if b < a { b } else { a })
                .unwrap_or(0.0),
            FunctionType::Max => arguments
                .iter()
                .copied()
                .reduce(|a, b| if b > a { b } else { a })
                .unwrap_or(0.0),
            FunctionType::Round => arguments.last().map_or(0.0, |v| v.round()),
            FunctionType::Ceil => arguments.last().map_or(0.0, |v| v.ceil()),
            FunctionType::Floor => arguments.last().map_or(0.0, |v| v.floor()),
            FunctionType::Sqrt => arguments.last().map_or(0.0, |v| v.sqrt()),
            FunctionType::Pow if arguments.len() > 1 => arguments
                .last()
                .unwrap()
                .powf(arguments[arguments.len() - 2]),
            FunctionType::Exp => arguments.last().map_or(0.0, |v| v.exp()),
            FunctionType::Log => arguments.last().map_or(0.0, |v| v.ln()),
            FunctionType::Cosine => arguments.last().map_or(0.0, |v| v.cos()),
            FunctionType::Sine => arguments.last().map_or(0.0, |v| v.sin()),
            FunctionType::Tangent => arguments.last().map_or(0.0, |v| v.tan()),
            FunctionType::Acosine => arguments.last().map_or(0.0, |v| v.acos()),
            FunctionType::Asine => arguments.last().map_or(0.0, |v| v.asin()),
            FunctionType::Atangent => arguments.last().map_or(0.0, |v| v.atan()),
            FunctionType::Atangent2 if arguments.len() > 1 => arguments
                .last()
                .unwrap()
                .atan2(arguments[arguments.len() - 2]),
            FunctionType::Random => {
                let random = self.get_random(0);
                let (lower, upper) = match arguments.len() {
                    0 => (0.0, 1.0),
                    1 => (0.0, *arguments.last().unwrap()),
                    _ => (*arguments.last().unwrap(), arguments[arguments.len() - 2]),
                };
                lower + (upper - lower) * random
            }
            _ => 0.0,
        }
    }
    pub fn convert<'a>(&'a mut self, value: &dyn DataValue) -> &'a dyn DataValue {
        let input = value
            .as_any()
            .downcast_ref::<DataValueNumber>()
            .map(DataValueNumber::value)
            .or_else(|| {
                value
                    .as_any()
                    .downcast_ref::<DataValueSymbolListIndex>()
                    .map(|v| v.value() as f32)
            });
        let Some(input) = input else {
            self.output.set_value(DataValueNumber::DEFAULT_VALUE);
            return &self.output;
        };
        let mut result = input;
        let mut stack = Vec::new();
        let queue = self.output_queue.clone();
        for token in queue {
            match Self::token_kind(&token) {
                FormulaTokenKind::Operation(operation) if stack.len() > 1 => {
                    let right = stack.pop().unwrap();
                    let left = stack.pop().unwrap();
                    stack.push(Self::apply_operation(left, right, operation));
                }
                FormulaTokenKind::Function(function) => {
                    let count = self
                        .argument_counts
                        .get(&token.identity_key())
                        .copied()
                        .unwrap_or(0);
                    let value = self.apply_function(&mut stack, function, count);
                    stack.push(value);
                }
                FormulaTokenKind::Input => stack.push(input),
                FormulaTokenKind::Value(value) => stack.push(value),
                _ => {}
            }
        }
        if stack.len() == 1 {
            result = stack.pop().unwrap();
        }
        self.output.set_value(result);
        &self.output
    }
    pub fn reverse_convert<'a>(&'a mut self, value: &dyn DataValue) -> &'a dyn DataValue {
        self.convert(value)
    }
    pub fn add_token(&mut self, token: CoreHandle) {
        self.tokens.push(token)
    }
    pub fn add_output_token(&mut self, token: CoreHandle, arguments: i32) {
        self.argument_counts.insert(token.identity_key(), arguments);
        self.output_queue.push(token)
    }
    pub fn clone_definition(&self) -> Self {
        let mut cloned = Self::default();
        cloned
            .base
            .set_random_mode_value_value(self.base.random_mode_value());
        cloned
            .base
            .base
            .base
            .set_name_value(self.base.base.base.name().to_owned());
        cloned
    }
    pub fn complete_clone(source: &CoreHandle, cloned: &CoreHandle) -> bool {
        if !super::data_converter::DataConverter::complete_clone(source, cloned) {
            return false;
        }
        let Some(tokens) = source.with_downcast::<Self, _>(|source| {
            source
                .output_queue
                .iter()
                .map(|token| {
                    (
                        token.clone(),
                        source
                            .argument_counts
                            .get(&token.identity_key())
                            .copied()
                            .unwrap_or(0),
                    )
                })
                .collect::<Vec<_>>()
        }) else {
            return false;
        };
        let source_binds = source.data_bind_container().unwrap().data_binds();
        let cloned_binds = cloned.data_bind_container().unwrap().data_binds();
        for (token, count) in tokens {
            let Some(cloned_token) = token.clone_occurrence() else {
                return false;
            };
            cloned.with_downcast_mut::<Self, _>(|cloned| {
                cloned.add_output_token(cloned_token.clone(), count)
            });
            for (index, bind) in source_binds.iter().enumerate() {
                let target = bind
                    .with(|bind| bind.as_data_bind().unwrap().target())
                    .flatten();
                if target.as_ref() == Some(&token) {
                    cloned_binds[index].with_mut(|bind| {
                        bind.as_data_bind_mut()
                            .unwrap()
                            .set_target(Some(cloned_token.clone()));
                    });
                }
            }
        }
        cloned
            .with_downcast_mut::<Self, _>(|cloned| cloned.is_instance = true)
            .is_some()
    }
    pub fn bind_from_context(
        &mut self,
        data_context: RuntimeDataContextHandle,
        data_bind: CoreHandle,
    ) {
        self.base
            .base
            .bind_from_context(data_context, data_bind.clone());
        let source = data_bind
            .with(|data_bind| {
                data_bind
                    .as_data_bind()
                    .and_then(|data_bind| data_bind.source())
            })
            .flatten();
        self.source = source.clone();
        let Some(dependent) = self.base.base.base.base.handle() else {
            return;
        };
        if let Some(source) = source {
            source.with_mut(|source| {
                if let Some(source) = source.as_view_model_instance_value_mut() {
                    source.add_dependent(ValueDependentHandle::core(dependent));
                }
            });
        }
    }
    pub fn add_dirt(&mut self, _value: u32, _recurse: bool) {
        if self.base.random_mode_value() == RandomMode::SourceChange as u32 {
            self.randoms.clear();
        }
    }
    pub fn unbind(&mut self) {
        if let Some(source) = self.source.take() {
            let Some(dependent) = self.base.base.base.base.handle() else {
                return;
            };
            source.with_mut(|source| {
                if let Some(source) = source.as_view_model_instance_value_mut() {
                    source.remove_dependent(&ValueDependentHandle::core(dependent));
                }
            });
        }
    }
    pub fn set_is_instance(&mut self, value: bool) {
        self.is_instance = value
    }
}

impl Drop for DataConverterFormula {
    fn drop(&mut self) {
        self.unbind();
        self.output_queue.clear();
        self.tokens.clear();
    }
}

impl FormulaSource for ViewModelInstanceValue {
    fn add_dependent(&mut self, dependent: CoreHandle) {
        ViewModelInstanceValue::add_dependent(self, ValueDependentHandle::core(dependent));
    }

    fn remove_dependent(&mut self, dependent: &CoreHandle) {
        ViewModelInstanceValue::remove_dependent(
            self,
            &ValueDependentHandle::core(dependent.clone()),
        );
    }
}

impl crate::mechanical_port::source::data_bind::converters::formula::formula_token::DataConverterFormula
    for DataConverterFormula
{
    fn add_token(&mut self, token: CoreHandle) {
        self.add_token(token);
    }

    fn add_data_bind(&mut self, data_bind: CoreHandle) {
        self.base.base.add_data_bind(data_bind);
    }
}

impl crate::mechanical_port::source::generated::core_registry::DataConverterCapability
    for DataConverterFormula
{
    fn convert(
        &mut self,
        input: &dyn DataValue,
        _data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        output(Self::convert(self, input));
    }

    fn reverse_convert(
        &mut self,
        input: &dyn DataValue,
        _data_bind: &CoreHandle,
        output: &mut dyn FnMut(&dyn DataValue),
    ) {
        output(Self::reverse_convert(self, input));
    }

    fn output_type(&self) -> DataType {
        Self::output_type(self)
    }

    fn bind_context_handler(&self) -> crate::mechanical_port::source::data_bind::converters::data_converter::ConverterBindContextHandler{
        |owner, context, data_bind| {
            super::data_converter::DataConverter::bind_from_context_handle(
                owner,
                context,
                data_bind.clone(),
            );
            let source = data_bind
                .with(|bind| bind.as_data_bind().unwrap().source())
                .flatten();
            owner.with_downcast_mut::<Self, _>(|owner| owner.source = source.clone());
            if let Some(source) = source {
                source.with_mut(|source| {
                    source
                        .as_view_model_instance_value_mut()
                        .unwrap()
                        .add_dependent(ValueDependentHandle::core(owner.clone()))
                });
            }
        }
    }

    fn unbind(&mut self) {
        Self::unbind(self);
    }

    fn update(&mut self) {
        self.base.base.update();
    }

    fn reset(&mut self) {
        self.base.base.reset();
    }

    fn advance(&mut self, elapsed: f32) -> bool {
        self.base.base.advance(elapsed)
    }
}
