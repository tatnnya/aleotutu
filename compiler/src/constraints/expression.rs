//! Methods to enforce constraints on expressions in a resolved Leo program.

use crate::{
    constraints::{
        boolean::{enforce_and, enforce_or, evaluate_not, new_bool_constant},
        new_scope,
        ConstrainedCircuitMember,
        ConstrainedProgram,
        ConstrainedValue,
    },
    errors::ExpressionError,
    FieldType,
    GroupType,
};
use leo_types::{
    CircuitFieldDefinition,
    CircuitMember,
    Expression,
    Identifier,
    Integer,
    IntegerType,
    RangeOrExpression,
    Span,
    SpreadOrExpression,
    Type,
};

use snarkos_models::{
    curves::{Field, PrimeField},
    gadgets::{
        r1cs::ConstraintSystem,
        utilities::{boolean::Boolean, eq::EvaluateEqGadget, select::CondSelectGadget},
    },
};

impl<F: Field + PrimeField, G: GroupType<F>> ConstrainedProgram<F, G> {
    /// Enforce a variable expression by getting the resolved value
    pub(crate) fn evaluate_identifier(
        &mut self,
        file_scope: String,
        function_scope: String,
        expected_types: &Vec<Type>,
        unresolved_identifier: Identifier,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        // Evaluate the identifier name in the current function scope
        let variable_name = new_scope(function_scope, unresolved_identifier.to_string());
        let identifier_name = new_scope(file_scope, unresolved_identifier.to_string());

        let mut result_value = if let Some(value) = self.get(&variable_name) {
            // Reassigning variable to another variable
            value.clone()
        } else if let Some(value) = self.get(&identifier_name) {
            // Check global scope (function and circuit names)
            value.clone()
        } else if let Some(value) = self.get(&unresolved_identifier.name) {
            // Check imported file scope
            value.clone()
        } else {
            return Err(ExpressionError::undefined_identifier(unresolved_identifier));
        };

        result_value.resolve_type(expected_types, unresolved_identifier.span.clone())?;

        Ok(result_value)
    }

    /// Enforce numerical operations
    fn enforce_add_expression<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            (ConstrainedValue::Integer(num_1), ConstrainedValue::Integer(num_2)) => {
                Ok(ConstrainedValue::Integer(num_1.add(cs, num_2, span)?))
            }
            (ConstrainedValue::Field(fe_1), ConstrainedValue::Field(fe_2)) => {
                Ok(ConstrainedValue::Field(fe_1.add(cs, &fe_2, span)?))
            }
            (ConstrainedValue::Group(ge_1), ConstrainedValue::Group(ge_2)) => {
                Ok(ConstrainedValue::Group(ge_1.add(cs, &ge_2, span)?))
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2, span.clone())?;
                self.enforce_add_expression(cs, val_1, val_2, span)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1, span.clone())?;
                self.enforce_add_expression(cs, val_1, val_2, span)
            }
            (val_1, val_2) => Err(ExpressionError::incompatible_types(
                format!("{} + {}", val_1, val_2),
                span,
            )),
        }
    }

    fn enforce_sub_expression<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            (ConstrainedValue::Integer(num_1), ConstrainedValue::Integer(num_2)) => {
                Ok(ConstrainedValue::Integer(num_1.sub(cs, num_2, span)?))
            }
            (ConstrainedValue::Field(fe_1), ConstrainedValue::Field(fe_2)) => {
                Ok(ConstrainedValue::Field(fe_1.sub(cs, &fe_2, span)?))
            }
            (ConstrainedValue::Group(ge_1), ConstrainedValue::Group(ge_2)) => {
                Ok(ConstrainedValue::Group(ge_1.sub(cs, &ge_2, span)?))
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2, span.clone())?;
                self.enforce_sub_expression(cs, val_1, val_2, span)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1, span.clone())?;
                self.enforce_sub_expression(cs, val_1, val_2, span)
            }
            (val_1, val_2) => Err(ExpressionError::incompatible_types(
                format!("{} - {}", val_1, val_2),
                span,
            )),
        }
    }

    fn enforce_mul_expression<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            (ConstrainedValue::Integer(num_1), ConstrainedValue::Integer(num_2)) => {
                Ok(ConstrainedValue::Integer(num_1.mul(cs, num_2, span)?))
            }
            (ConstrainedValue::Field(fe_1), ConstrainedValue::Field(fe_2)) => {
                Ok(ConstrainedValue::Field(fe_1.mul(cs, &fe_2, span)?))
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2, span.clone())?;
                self.enforce_mul_expression(cs, val_1, val_2, span)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1, span.clone())?;
                self.enforce_mul_expression(cs, val_1, val_2, span)
            }
            (val_1, val_2) => {
                return Err(ExpressionError::incompatible_types(
                    format!("{} * {}", val_1, val_2),
                    span,
                ));
            }
        }
    }

    fn enforce_div_expression<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            (ConstrainedValue::Integer(num_1), ConstrainedValue::Integer(num_2)) => {
                Ok(ConstrainedValue::Integer(num_1.div(cs, num_2, span)?))
            }
            (ConstrainedValue::Field(fe_1), ConstrainedValue::Field(fe_2)) => {
                Ok(ConstrainedValue::Field(fe_1.div(cs, &fe_2, span)?))
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2, span.clone())?;
                self.enforce_div_expression(cs, val_1, val_2, span)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1, span.clone())?;
                self.enforce_div_expression(cs, val_1, val_2, span)
            }
            (val_1, val_2) => {
                return Err(ExpressionError::incompatible_types(
                    format!("{} / {}", val_1, val_2,),
                    span,
                ));
            }
        }
    }

    fn enforce_pow_expression<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            (ConstrainedValue::Integer(num_1), ConstrainedValue::Integer(num_2)) => {
                Ok(ConstrainedValue::Integer(num_1.pow(cs, num_2, span)?))
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2, span.clone())?;
                self.enforce_pow_expression(cs, val_1, val_2, span)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1, span.clone())?;
                self.enforce_pow_expression(cs, val_1, val_2, span)
            }
            (val_1, val_2) => Err(ExpressionError::incompatible_types(
                format!("{} ** {}", val_1, val_2,),
                span,
            )),
        }
    }

    /// Evaluate Boolean operations
    fn evaluate_eq_expression<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        let mut unique_namespace = cs.ns(|| format!("evaluate {} == {} {}:{}", left, right, span.line, span.start));
        let constraint_result = match (left, right) {
            (ConstrainedValue::Boolean(bool_1), ConstrainedValue::Boolean(bool_2)) => {
                bool_1.evaluate_equal(unique_namespace, &bool_2)
            }
            (ConstrainedValue::Integer(num_1), ConstrainedValue::Integer(num_2)) => {
                num_1.evaluate_equal(unique_namespace, &num_2)
            }
            (ConstrainedValue::Field(fe_1), ConstrainedValue::Field(fe_2)) => {
                fe_1.evaluate_equal(unique_namespace, &fe_2)
            }
            (ConstrainedValue::Group(ge_1), ConstrainedValue::Group(ge_2)) => {
                ge_1.evaluate_equal(unique_namespace, &ge_2)
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2, span.clone())?;
                return self.evaluate_eq_expression(&mut unique_namespace, val_1, val_2, span);
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1, span.clone())?;
                return self.evaluate_eq_expression(&mut unique_namespace, val_1, val_2, span);
            }
            (val_1, val_2) => {
                return Err(ExpressionError::incompatible_types(
                    format!("{} == {}", val_1, val_2,),
                    span,
                ));
            }
        };

        let boolean =
            constraint_result.map_err(|e| ExpressionError::cannot_enforce(format!("evaluate equals"), e, span))?;

        Ok(ConstrainedValue::Boolean(boolean))
    }

    //TODO: unsafe for allocated values
    fn evaluate_ge_expression(
        &mut self,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            (ConstrainedValue::Integer(num_1), ConstrainedValue::Integer(num_2)) => {
                let result = num_1.ge(&num_2);
                Ok(ConstrainedValue::Boolean(Boolean::Constant(result)))
            }
            (ConstrainedValue::Field(fe_1), ConstrainedValue::Field(fe_2)) => {
                let result = fe_1.ge(&fe_2);
                Ok(ConstrainedValue::Boolean(Boolean::Constant(result)))
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2, span.clone())?;
                self.evaluate_ge_expression(val_1, val_2, span)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1, span.clone())?;
                self.evaluate_ge_expression(val_1, val_2, span)
            }
            (val_1, val_2) => Err(ExpressionError::incompatible_types(
                format!("{} >= {}", val_1, val_2),
                span,
            )),
        }
    }

    //TODO: unsafe for allocated values
    fn evaluate_gt_expression(
        &mut self,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            (ConstrainedValue::Integer(num_1), ConstrainedValue::Integer(num_2)) => {
                let result = num_1.gt(&num_2);
                Ok(ConstrainedValue::Boolean(Boolean::Constant(result)))
            }
            (ConstrainedValue::Field(fe_1), ConstrainedValue::Field(fe_2)) => {
                let result = fe_1.gt(&fe_2);
                Ok(ConstrainedValue::Boolean(Boolean::Constant(result)))
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2, span.clone())?;
                self.evaluate_gt_expression(val_1, val_2, span)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1, span.clone())?;
                self.evaluate_gt_expression(val_1, val_2, span)
            }
            (val_1, val_2) => Err(ExpressionError::incompatible_types(
                format!("{} > {}", val_1, val_2),
                span,
            )),
        }
    }

    //TODO: unsafe for allocated values
    fn evaluate_le_expression(
        &mut self,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            (ConstrainedValue::Integer(num_1), ConstrainedValue::Integer(num_2)) => {
                let result = num_1.le(&num_2);
                Ok(ConstrainedValue::Boolean(Boolean::Constant(result)))
            }
            (ConstrainedValue::Field(fe_1), ConstrainedValue::Field(fe_2)) => {
                let result = fe_1.le(&fe_2);
                Ok(ConstrainedValue::Boolean(Boolean::Constant(result)))
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2, span.clone())?;
                self.evaluate_le_expression(val_1, val_2, span)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1, span.clone())?;
                self.evaluate_le_expression(val_1, val_2, span)
            }
            (val_1, val_2) => Err(ExpressionError::incompatible_types(
                format!("{} <= {}", val_1, val_2),
                span,
            )),
        }
    }

    //TODO: unsafe for allocated values
    fn evaluate_lt_expression(
        &mut self,
        left: ConstrainedValue<F, G>,
        right: ConstrainedValue<F, G>,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match (left, right) {
            (ConstrainedValue::Integer(num_1), ConstrainedValue::Integer(num_2)) => {
                let result = num_1.lt(&num_2);
                Ok(ConstrainedValue::Boolean(Boolean::Constant(result)))
            }
            (ConstrainedValue::Field(fe_1), ConstrainedValue::Field(fe_2)) => {
                let result = fe_1.lt(&fe_2);
                Ok(ConstrainedValue::Boolean(Boolean::Constant(result)))
            }
            (ConstrainedValue::Unresolved(string), val_2) => {
                let val_1 = ConstrainedValue::from_other(string, &val_2, span.clone())?;
                self.evaluate_lt_expression(val_1, val_2, span)
            }
            (val_1, ConstrainedValue::Unresolved(string)) => {
                let val_2 = ConstrainedValue::from_other(string, &val_1, span.clone())?;
                self.evaluate_lt_expression(val_1, val_2, span)
            }
            (val_1, val_2) => Err(ExpressionError::incompatible_types(
                format!("{} < {}", val_1, val_2,),
                span,
            )),
        }
    }

    /// Enforce ternary conditional expression
    fn enforce_conditional_expression<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        expected_types: &Vec<Type>,
        first: Expression,
        second: Expression,
        third: Expression,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        let resolved_first = match self.enforce_expression(
            cs,
            file_scope.clone(),
            function_scope.clone(),
            &vec![Type::Boolean],
            first,
        )? {
            ConstrainedValue::Boolean(resolved) => resolved,
            value => return Err(ExpressionError::conditional_boolean(value.to_string(), span)),
        };

        let resolved_second = self.enforce_expression_value(
            cs,
            file_scope.clone(),
            function_scope.clone(),
            expected_types,
            second,
            span.clone(),
        )?;
        let resolved_third =
            self.enforce_expression_value(cs, file_scope, function_scope, expected_types, third, span.clone())?;

        let unique_namespace = cs.ns(|| {
            format!(
                "select {} or {} {}:{}",
                resolved_second, resolved_third, span.line, span.start
            )
        });

        match (resolved_second, resolved_third) {
            (ConstrainedValue::Boolean(bool_2), ConstrainedValue::Boolean(bool_3)) => {
                let result = Boolean::conditionally_select(unique_namespace, &resolved_first, &bool_2, &bool_3)
                    .map_err(|e| ExpressionError::cannot_enforce(format!("conditional select"), e, span))?;

                Ok(ConstrainedValue::Boolean(result))
            }
            (ConstrainedValue::Integer(integer_2), ConstrainedValue::Integer(integer_3)) => {
                let result = Integer::conditionally_select(unique_namespace, &resolved_first, &integer_2, &integer_3)
                    .map_err(|e| ExpressionError::cannot_enforce(format!("conditional select"), e, span))?;

                Ok(ConstrainedValue::Integer(result))
            }
            (ConstrainedValue::Field(field_1), ConstrainedValue::Field(field_2)) => {
                let result = FieldType::conditionally_select(unique_namespace, &resolved_first, &field_1, &field_2)
                    .map_err(|e| ExpressionError::cannot_enforce(format!("conditional select"), e, span))?;

                Ok(ConstrainedValue::Field(result))
            }
            (ConstrainedValue::Group(point_1), ConstrainedValue::Group(point_2)) => {
                let result = G::conditionally_select(unique_namespace, &resolved_first, &point_1, &point_2)
                    .map_err(|e| ExpressionError::cannot_enforce(format!("conditional select"), e, span))?;

                Ok(ConstrainedValue::Group(result))
            }
            (val_1, val_2) => Err(ExpressionError::incompatible_types(
                format!("ternary between {} and {}", val_1, val_2),
                span,
            )),
        }
    }

    /// Enforce array expressions
    fn enforce_array_expression<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        expected_types: &Vec<Type>,
        array: Vec<Box<SpreadOrExpression>>,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        // Check explicit array type dimension if given
        let mut expected_types = expected_types.clone();
        let expected_dimensions = vec![];

        if !expected_types.is_empty() {
            match expected_types[0] {
                Type::Array(ref _type, ref dimensions) => {
                    expected_types = vec![expected_types[0].inner_dimension(dimensions)];
                }
                ref _type => {
                    return Err(ExpressionError::unexpected_array(
                        expected_types[0].to_string(),
                        _type.to_string(),
                        span,
                    ));
                }
            }
        }

        let mut result = vec![];
        for element in array.into_iter() {
            match *element {
                SpreadOrExpression::Spread(spread) => match spread {
                    Expression::Identifier(identifier) => {
                        let array_name = new_scope(function_scope.clone(), identifier.to_string());
                        match self.get(&array_name) {
                            Some(value) => match value {
                                ConstrainedValue::Array(array) => result.extend(array.clone()),
                                value => {
                                    return Err(ExpressionError::invalid_spread(value.to_string(), span));
                                }
                            },
                            None => return Err(ExpressionError::undefined_array(identifier.name, span)),
                        }
                    }
                    value => return Err(ExpressionError::invalid_spread(value.to_string(), span)),
                },
                SpreadOrExpression::Expression(expression) => {
                    result.push(self.enforce_expression(
                        cs,
                        file_scope.clone(),
                        function_scope.clone(),
                        &expected_types,
                        expression,
                    )?);
                }
            }
        }

        // Check expected_dimensions if given
        if !expected_dimensions.is_empty() {
            if expected_dimensions[expected_dimensions.len() - 1] != result.len() {
                return Err(ExpressionError::invalid_length(
                    expected_dimensions[expected_dimensions.len() - 1],
                    result.len(),
                    span,
                ));
            }
        }

        Ok(ConstrainedValue::Array(result))
    }

    pub(crate) fn enforce_index<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        index: Expression,
        span: Span,
    ) -> Result<usize, ExpressionError> {
        let expected_types = vec![Type::IntegerType(IntegerType::U32)];
        match self.enforce_expression_value(
            cs,
            file_scope.clone(),
            function_scope.clone(),
            &expected_types,
            index,
            span.clone(),
        )? {
            ConstrainedValue::Integer(number) => Ok(number.to_usize(span.clone())?),
            value => Err(ExpressionError::invalid_index(value.to_string(), span)),
        }
    }

    fn enforce_array_access_expression<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        expected_types: &Vec<Type>,
        array: Box<Expression>,
        index: RangeOrExpression,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        let array = match self.enforce_expression_value(
            cs,
            file_scope.clone(),
            function_scope.clone(),
            expected_types,
            *array,
            span.clone(),
        )? {
            ConstrainedValue::Array(array) => array,
            value => return Err(ExpressionError::undefined_array(value.to_string(), span)),
        };

        match index {
            RangeOrExpression::Range(from, to) => {
                let from_resolved = match from {
                    Some(from_index) => {
                        self.enforce_index(cs, file_scope.clone(), function_scope.clone(), from_index, span.clone())?
                    }
                    None => 0usize, // Array slice starts at index 0
                };
                let to_resolved = match to {
                    Some(to_index) => {
                        self.enforce_index(cs, file_scope.clone(), function_scope.clone(), to_index, span.clone())?
                    }
                    None => array.len(), // Array slice ends at array length
                };
                Ok(ConstrainedValue::Array(array[from_resolved..to_resolved].to_owned()))
            }
            RangeOrExpression::Expression(index) => {
                let index_resolved = self.enforce_index(cs, file_scope, function_scope, index, span)?;
                Ok(array[index_resolved].to_owned())
            }
        }
    }

    fn enforce_circuit_expression<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        identifier: Identifier,
        members: Vec<CircuitFieldDefinition>,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        let mut program_identifier = new_scope(file_scope.clone(), identifier.to_string());

        if identifier.is_self() {
            program_identifier = file_scope.clone();
        }

        let circuit = match self.get(&program_identifier) {
            Some(value) => value.clone().extract_circuit(span.clone())?,
            None => return Err(ExpressionError::undefined_circuit(identifier.to_string(), span)),
        };

        let circuit_identifier = circuit.circuit_name.clone();
        let mut resolved_members = vec![];

        for member in circuit.members.clone().into_iter() {
            match member {
                CircuitMember::CircuitField(identifier, _type) => {
                    let matched_field = members
                        .clone()
                        .into_iter()
                        .find(|field| field.identifier.eq(&identifier));
                    match matched_field {
                        Some(field) => {
                            // Resolve and enforce circuit object
                            let field_value = self.enforce_expression(
                                cs,
                                file_scope.clone(),
                                function_scope.clone(),
                                &vec![_type.clone()],
                                field.expression,
                            )?;

                            resolved_members.push(ConstrainedCircuitMember(identifier, field_value))
                        }
                        None => return Err(ExpressionError::expected_circuit_member(identifier.to_string(), span)),
                    }
                }
                CircuitMember::CircuitFunction(_static, function) => {
                    let identifier = function.function_name.clone();
                    let mut constrained_function_value =
                        ConstrainedValue::Function(Some(circuit_identifier.clone()), function);

                    if _static {
                        constrained_function_value = ConstrainedValue::Static(Box::new(constrained_function_value));
                    }

                    resolved_members.push(ConstrainedCircuitMember(identifier, constrained_function_value));
                }
            };
        }

        Ok(ConstrainedValue::CircuitExpression(
            circuit_identifier.clone(),
            resolved_members,
        ))
    }

    fn enforce_circuit_access_expression<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        expected_types: &Vec<Type>,
        circuit_identifier: Box<Expression>,
        circuit_member: Identifier,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        let (circuit_name, members) = match self.enforce_expression_value(
            cs,
            file_scope.clone(),
            function_scope.clone(),
            expected_types,
            *circuit_identifier.clone(),
            span.clone(),
        )? {
            ConstrainedValue::CircuitExpression(name, members) => (name, members),
            value => return Err(ExpressionError::undefined_circuit(value.to_string(), span)),
        };

        let matched_member = members.clone().into_iter().find(|member| member.0 == circuit_member);

        match matched_member {
            Some(member) => {
                match &member.1 {
                    ConstrainedValue::Function(ref _circuit_identifier, ref _function) => {
                        // Pass static circuit fields into function call by value
                        for stored_member in members {
                            match &stored_member.1 {
                                ConstrainedValue::Function(_, _) => {}
                                ConstrainedValue::Static(_) => {}
                                _ => {
                                    let circuit_scope = new_scope(file_scope.clone(), circuit_name.to_string());
                                    let function_scope = new_scope(circuit_scope, member.0.to_string());
                                    let field = new_scope(function_scope, stored_member.0.to_string());

                                    self.store(field, stored_member.1.clone());
                                }
                            }
                        }
                    }
                    ConstrainedValue::Static(value) => {
                        return Err(ExpressionError::invalid_static_access(value.to_string(), span));
                    }
                    _ => {}
                }
                Ok(member.1)
            }
            None => Err(ExpressionError::undefined_member_access(
                circuit_name.to_string(),
                circuit_member.to_string(),
                span,
            )),
        }
    }

    fn enforce_circuit_static_access_expression<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        expected_types: &Vec<Type>,
        circuit_identifier: Box<Expression>,
        circuit_member: Identifier,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        // Get defined circuit
        let circuit = match self.enforce_expression(
            cs,
            file_scope.clone(),
            function_scope.clone(),
            expected_types,
            *circuit_identifier.clone(),
        )? {
            ConstrainedValue::CircuitDefinition(circuit_definition) => circuit_definition,
            value => return Err(ExpressionError::undefined_circuit(value.to_string(), span)),
        };

        // Find static circuit function
        let matched_function = circuit.members.into_iter().find(|member| match member {
            CircuitMember::CircuitFunction(_static, function) => function.function_name == circuit_member,
            _ => false,
        });

        // Return errors if no static function exists
        let function = match matched_function {
            Some(CircuitMember::CircuitFunction(_static, function)) => {
                if _static {
                    function
                } else {
                    return Err(ExpressionError::invalid_member_access(
                        function.function_name.to_string(),
                        span,
                    ));
                }
            }
            _ => {
                return Err(ExpressionError::undefined_member_access(
                    circuit.circuit_name.to_string(),
                    circuit_member.to_string(),
                    span,
                ));
            }
        };

        Ok(ConstrainedValue::Function(Some(circuit.circuit_name), function))
    }

    fn enforce_function_call_expression<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        expected_types: &Vec<Type>,
        function: Box<Expression>,
        arguments: Vec<Expression>,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        let function_value = self.enforce_expression(
            cs,
            file_scope.clone(),
            function_scope.clone(),
            expected_types,
            *function.clone(),
        )?;

        let (outer_scope, function_call) = function_value.extract_function(file_scope, span.clone())?;

        let name_unique = format!(
            "function call {} {}:{}",
            function_call.get_name(),
            span.line,
            span.start,
        );

        match self.enforce_function(
            &mut cs.ns(|| name_unique),
            outer_scope,
            function_scope,
            function_call,
            arguments,
        ) {
            Ok(ConstrainedValue::Return(return_values)) => {
                if return_values.len() == 1 {
                    Ok(return_values[0].clone())
                } else {
                    Ok(ConstrainedValue::Return(return_values))
                }
            }
            Ok(_) => Err(ExpressionError::function_no_return(function.to_string(), span)),
            Err(error) => Err(ExpressionError::from(Box::new(error))),
        }
    }

    pub(crate) fn enforce_number_implicit(
        expected_types: &Vec<Type>,
        value: String,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        if expected_types.len() == 1 {
            return Ok(ConstrainedValue::from_type(value, &expected_types[0], span)?);
        }

        Ok(ConstrainedValue::Unresolved(value))
    }

    /// Enforce a branch of a binary expression.
    /// We don't care about mutability because we are not changing any variables.
    /// We try to resolve unresolved types here if the type is given explicitly.
    pub(crate) fn enforce_expression_value<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        expected_types: &Vec<Type>,
        expression: Expression,
        span: Span,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        let mut branch = self.enforce_expression(cs, file_scope, function_scope, expected_types, expression)?;

        branch.get_inner_mut();
        branch.resolve_type(expected_types, span)?;

        Ok(branch)
    }

    pub(crate) fn enforce_binary_expression<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        expected_types: &Vec<Type>,
        left: Expression,
        right: Expression,
        span: Span,
    ) -> Result<(ConstrainedValue<F, G>, ConstrainedValue<F, G>), ExpressionError> {
        let mut resolved_left = self.enforce_expression_value(
            cs,
            file_scope.clone(),
            function_scope.clone(),
            expected_types,
            left,
            span.clone(),
        )?;
        let mut resolved_right = self.enforce_expression_value(
            cs,
            file_scope.clone(),
            function_scope.clone(),
            expected_types,
            right,
            span.clone(),
        )?;

        resolved_left.resolve_types(&mut resolved_right, expected_types, span)?;

        Ok((resolved_left, resolved_right))
    }

    pub(crate) fn enforce_expression<CS: ConstraintSystem<F>>(
        &mut self,
        cs: &mut CS,
        file_scope: String,
        function_scope: String,
        expected_types: &Vec<Type>,
        expression: Expression,
    ) -> Result<ConstrainedValue<F, G>, ExpressionError> {
        match expression {
            // Variables
            Expression::Identifier(unresolved_variable) => {
                self.evaluate_identifier(file_scope, function_scope, expected_types, unresolved_variable)
            }

            // Values
            Expression::Integer(type_, integer, span) => {
                Ok(ConstrainedValue::Integer(Integer::new_constant(&type_, integer, span)?))
            }
            Expression::Field(field, span) => Ok(ConstrainedValue::Field(FieldType::constant(field, span)?)),
            Expression::Group(group_affine, span) => Ok(ConstrainedValue::Group(G::constant(group_affine, span)?)),
            Expression::Boolean(boolean, span) => Ok(ConstrainedValue::Boolean(new_bool_constant(boolean, span)?)),
            Expression::Implicit(value, span) => Self::enforce_number_implicit(expected_types, value, span),

            // Binary operations
            Expression::Add(left, right, span) => {
                let (resolved_left, resolved_right) = self.enforce_binary_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *left,
                    *right,
                    span.clone(),
                )?;

                self.enforce_add_expression(cs, resolved_left, resolved_right, span)
            }
            Expression::Sub(left, right, span) => {
                let (resolved_left, resolved_right) = self.enforce_binary_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *left,
                    *right,
                    span.clone(),
                )?;

                self.enforce_sub_expression(cs, resolved_left, resolved_right, span)
            }
            Expression::Mul(left, right, span) => {
                let (resolved_left, resolved_right) = self.enforce_binary_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *left,
                    *right,
                    span.clone(),
                )?;

                self.enforce_mul_expression(cs, resolved_left, resolved_right, span)
            }
            Expression::Div(left, right, span) => {
                let (resolved_left, resolved_right) = self.enforce_binary_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *left,
                    *right,
                    span.clone(),
                )?;

                self.enforce_div_expression(cs, resolved_left, resolved_right, span)
            }
            Expression::Pow(left, right, span) => {
                let (resolved_left, resolved_right) = self.enforce_binary_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *left,
                    *right,
                    span.clone(),
                )?;

                self.enforce_pow_expression(cs, resolved_left, resolved_right, span)
            }

            // Boolean operations
            Expression::Not(expression, span) => Ok(evaluate_not(
                self.enforce_expression(cs, file_scope, function_scope, expected_types, *expression)?,
                span,
            )?),
            Expression::Or(left, right, span) => {
                let (resolved_left, resolved_right) = self.enforce_binary_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *left,
                    *right,
                    span.clone(),
                )?;

                Ok(enforce_or(cs, resolved_left, resolved_right, span)?)
            }
            Expression::And(left, right, span) => {
                let (resolved_left, resolved_right) = self.enforce_binary_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    expected_types,
                    *left,
                    *right,
                    span.clone(),
                )?;

                Ok(enforce_and(cs, resolved_left, resolved_right, span)?)
            }
            Expression::Eq(left, right, span) => {
                let (resolved_left, resolved_right) = self.enforce_binary_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    &vec![],
                    *left,
                    *right,
                    span.clone(),
                )?;

                Ok(self.evaluate_eq_expression(cs, resolved_left, resolved_right, span)?)
            }
            Expression::Ge(left, right, span) => {
                let (resolved_left, resolved_right) = self.enforce_binary_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    &vec![],
                    *left,
                    *right,
                    span.clone(),
                )?;

                Ok(self.evaluate_ge_expression(resolved_left, resolved_right, span)?)
            }
            Expression::Gt(left, right, span) => {
                let (resolved_left, resolved_right) = self.enforce_binary_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    &vec![],
                    *left,
                    *right,
                    span.clone(),
                )?;

                Ok(self.evaluate_gt_expression(resolved_left, resolved_right, span)?)
            }
            Expression::Le(left, right, span) => {
                let (resolved_left, resolved_right) = self.enforce_binary_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    &vec![],
                    *left,
                    *right,
                    span.clone(),
                )?;

                Ok(self.evaluate_le_expression(resolved_left, resolved_right, span)?)
            }
            Expression::Lt(left, right, span) => {
                let (resolved_left, resolved_right) = self.enforce_binary_expression(
                    cs,
                    file_scope.clone(),
                    function_scope.clone(),
                    &vec![],
                    *left,
                    *right,
                    span.clone(),
                )?;

                Ok(self.evaluate_lt_expression(resolved_left, resolved_right, span)?)
            }

            // Conditionals
            Expression::IfElse(first, second, third, span) => self.enforce_conditional_expression(
                cs,
                file_scope,
                function_scope,
                expected_types,
                *first,
                *second,
                *third,
                span,
            ),

            // Arrays
            Expression::Array(array, span) => {
                self.enforce_array_expression(cs, file_scope, function_scope, expected_types, array, span)
            }
            Expression::ArrayAccess(array, index, span) => self.enforce_array_access_expression(
                cs,
                file_scope,
                function_scope,
                expected_types,
                array,
                *index,
                span,
            ),

            // Circuits
            Expression::Circuit(circuit_name, members, span) => {
                self.enforce_circuit_expression(cs, file_scope, function_scope, circuit_name, members, span)
            }
            Expression::CircuitMemberAccess(circuit_variable, circuit_member, span) => self
                .enforce_circuit_access_expression(
                    cs,
                    file_scope,
                    function_scope,
                    expected_types,
                    circuit_variable,
                    circuit_member,
                    span,
                ),
            Expression::CircuitStaticFunctionAccess(circuit_identifier, circuit_member, span) => self
                .enforce_circuit_static_access_expression(
                    cs,
                    file_scope,
                    function_scope,
                    expected_types,
                    circuit_identifier,
                    circuit_member,
                    span,
                ),

            // Functions
            Expression::FunctionCall(function, arguments, span) => self.enforce_function_call_expression(
                cs,
                file_scope,
                function_scope,
                expected_types,
                function,
                arguments,
                span,
            ),
        }
    }
}